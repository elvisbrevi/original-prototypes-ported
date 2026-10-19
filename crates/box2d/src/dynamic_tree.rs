// b2DynamicTree: árbol de AABB "gordos" para la fase amplia. Se traduce entero (asignación de
// nodos con lista libre, inserción por coste de perímetro y rotaciones de balanceo) porque los
// identificadores de los proxies deciden el orden de las parejas de contacto.

use crate::collision::{Aabb, test_overlap_aabb};
use crate::math::{Vec2, vec2};
use crate::settings::{AABB_EXTENSION, AABB_MULTIPLIER};

pub const NULL_NODE: i32 = -1;

#[derive(Clone, Copy, Debug)]
pub struct TreeNode {
    pub aabb: Aabb,
    pub user_data: usize,
    /// parent o, en la lista libre, next.
    pub parent: i32,
    pub child1: i32,
    pub child2: i32,
    pub height: i32,
}

impl TreeNode {
    fn vacio() -> Self {
        Self {
            aabb: Aabb::default(),
            user_data: 0,
            parent: NULL_NODE,
            child1: 0,
            child2: 0,
            height: -1,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.child1 == NULL_NODE
    }
}

pub struct DynamicTree {
    root: i32,
    nodes: Vec<TreeNode>,
    node_count: i32,
    free_list: i32,
}

impl Default for DynamicTree {
    fn default() -> Self {
        let capacity = 16;
        let mut nodes = vec![TreeNode::vacio(); capacity];
        for (i, n) in nodes.iter_mut().enumerate() {
            n.parent = if i + 1 < capacity {
                i as i32 + 1
            } else {
                NULL_NODE
            };
            n.height = -1;
            n.child1 = 0;
            n.child2 = 0;
        }
        Self {
            root: NULL_NODE,
            nodes,
            node_count: 0,
            free_list: 0,
        }
    }
}

impl DynamicTree {
    fn n(&self, i: i32) -> &TreeNode {
        &self.nodes[i as usize]
    }

    fn m(&mut self, i: i32) -> &mut TreeNode {
        &mut self.nodes[i as usize]
    }

    fn allocate_node(&mut self) -> i32 {
        if self.free_list == NULL_NODE {
            let old = self.nodes.len();
            let capacity = old * 2;
            self.nodes.resize(capacity, TreeNode::vacio());
            for i in old..capacity {
                self.nodes[i].parent = if i + 1 < capacity {
                    i as i32 + 1
                } else {
                    NULL_NODE
                };
                self.nodes[i].height = -1;
            }
            self.free_list = self.node_count;
        }
        let node_id = self.free_list;
        self.free_list = self.n(node_id).parent;
        let node = self.m(node_id);
        node.parent = NULL_NODE;
        node.child1 = NULL_NODE;
        node.child2 = NULL_NODE;
        node.height = 0;
        node.user_data = 0;
        self.node_count += 1;
        node_id
    }

    fn free_node(&mut self, node_id: i32) {
        let free = self.free_list;
        let node = self.m(node_id);
        node.parent = free;
        node.height = -1;
        self.free_list = node_id;
        self.node_count -= 1;
    }

    pub fn create_proxy(&mut self, aabb: &Aabb, user_data: usize) -> i32 {
        let proxy_id = self.allocate_node();
        let r = vec2(AABB_EXTENSION, AABB_EXTENSION);
        let node = self.m(proxy_id);
        node.aabb.lower_bound = aabb.lower_bound - r;
        node.aabb.upper_bound = aabb.upper_bound + r;
        node.user_data = user_data;
        node.height = 0;
        self.insert_leaf(proxy_id);
        proxy_id
    }

    pub fn destroy_proxy(&mut self, proxy_id: i32) {
        self.remove_leaf(proxy_id);
        self.free_node(proxy_id);
    }

    pub fn move_proxy(&mut self, proxy_id: i32, aabb: &Aabb, displacement: Vec2) -> bool {
        if self.n(proxy_id).aabb.contains(aabb) {
            return false;
        }
        self.remove_leaf(proxy_id);
        let mut b = *aabb;
        let r = vec2(AABB_EXTENSION, AABB_EXTENSION);
        b.lower_bound -= r;
        b.upper_bound += r;
        let d = AABB_MULTIPLIER * displacement;
        if d.x < 0.0 {
            b.lower_bound.x += d.x;
        } else {
            b.upper_bound.x += d.x;
        }
        if d.y < 0.0 {
            b.lower_bound.y += d.y;
        } else {
            b.upper_bound.y += d.y;
        }
        self.m(proxy_id).aabb = b;
        self.insert_leaf(proxy_id);
        true
    }

    pub fn get_user_data(&self, proxy_id: i32) -> usize {
        self.n(proxy_id).user_data
    }

    pub fn get_fat_aabb(&self, proxy_id: i32) -> &Aabb {
        &self.n(proxy_id).aabb
    }

    /// Query: llama a `callback` con cada hoja que toca el AABB (en el orden de la pila).
    pub fn query(&self, aabb: &Aabb, mut callback: impl FnMut(i32) -> bool) {
        let mut stack: Vec<i32> = Vec::with_capacity(256);
        stack.push(self.root);
        while let Some(node_id) = stack.pop() {
            if node_id == NULL_NODE {
                continue;
            }
            let node = self.n(node_id);
            if test_overlap_aabb(&node.aabb, aabb) {
                if node.is_leaf() {
                    if !callback(node_id) {
                        return;
                    }
                } else {
                    stack.push(node.child1);
                    stack.push(node.child2);
                }
            }
        }
    }

    fn insert_leaf(&mut self, leaf: i32) {
        if self.root == NULL_NODE {
            self.root = leaf;
            self.m(leaf).parent = NULL_NODE;
            return;
        }
        let leaf_aabb = self.n(leaf).aabb;
        let mut index = self.root;
        while !self.n(index).is_leaf() {
            let child1 = self.n(index).child1;
            let child2 = self.n(index).child2;
            let area = self.n(index).aabb.get_perimeter();
            let combined = Aabb::combine(&self.n(index).aabb, &leaf_aabb);
            let combined_area = combined.get_perimeter();
            let cost = 2.0 * combined_area;
            let inheritance_cost = 2.0 * (combined_area - area);
            let coste_hijo = |t: &Self, c: i32| {
                let aabb = Aabb::combine(&leaf_aabb, &t.n(c).aabb);
                if t.n(c).is_leaf() {
                    aabb.get_perimeter() + inheritance_cost
                } else {
                    let old_area = t.n(c).aabb.get_perimeter();
                    let new_area = aabb.get_perimeter();
                    (new_area - old_area) + inheritance_cost
                }
            };
            let cost1 = coste_hijo(self, child1);
            let cost2 = coste_hijo(self, child2);
            if cost < cost1 && cost < cost2 {
                break;
            }
            index = if cost1 < cost2 { child1 } else { child2 };
        }
        let sibling = index;
        let old_parent = self.n(sibling).parent;
        let new_parent = self.allocate_node();
        let sibling_aabb = self.n(sibling).aabb;
        let sibling_height = self.n(sibling).height;
        {
            let np = self.m(new_parent);
            np.parent = old_parent;
            np.user_data = 0;
            np.aabb = Aabb::combine(&leaf_aabb, &sibling_aabb);
            np.height = sibling_height + 1;
        }
        if old_parent != NULL_NODE {
            if self.n(old_parent).child1 == sibling {
                self.m(old_parent).child1 = new_parent;
            } else {
                self.m(old_parent).child2 = new_parent;
            }
        } else {
            self.root = new_parent;
        }
        self.m(new_parent).child1 = sibling;
        self.m(new_parent).child2 = leaf;
        self.m(sibling).parent = new_parent;
        self.m(leaf).parent = new_parent;
        let mut index = self.n(leaf).parent;
        while index != NULL_NODE {
            index = self.balance(index);
            let child1 = self.n(index).child1;
            let child2 = self.n(index).child2;
            let height = 1 + self.n(child1).height.max(self.n(child2).height);
            let aabb = Aabb::combine(&self.n(child1).aabb, &self.n(child2).aabb);
            let node = self.m(index);
            node.height = height;
            node.aabb = aabb;
            index = node.parent;
        }
    }

    fn remove_leaf(&mut self, leaf: i32) {
        if leaf == self.root {
            self.root = NULL_NODE;
            return;
        }
        let parent = self.n(leaf).parent;
        let grand_parent = self.n(parent).parent;
        let sibling = if self.n(parent).child1 == leaf {
            self.n(parent).child2
        } else {
            self.n(parent).child1
        };
        if grand_parent != NULL_NODE {
            if self.n(grand_parent).child1 == parent {
                self.m(grand_parent).child1 = sibling;
            } else {
                self.m(grand_parent).child2 = sibling;
            }
            self.m(sibling).parent = grand_parent;
            self.free_node(parent);
            let mut index = grand_parent;
            while index != NULL_NODE {
                index = self.balance(index);
                let child1 = self.n(index).child1;
                let child2 = self.n(index).child2;
                let aabb = Aabb::combine(&self.n(child1).aabb, &self.n(child2).aabb);
                let height = 1 + self.n(child1).height.max(self.n(child2).height);
                let node = self.m(index);
                node.aabb = aabb;
                node.height = height;
                index = node.parent;
            }
        } else {
            self.root = sibling;
            self.m(sibling).parent = NULL_NODE;
            self.free_node(parent);
        }
    }

    /// Rotación izquierda o derecha si el nodo está desbalanceado; devuelve la nueva raíz.
    fn balance(&mut self, i_a: i32) -> i32 {
        let a = *self.n(i_a);
        if a.is_leaf() || a.height < 2 {
            return i_a;
        }
        let i_b = a.child1;
        let i_c = a.child2;
        let balance = self.n(i_c).height - self.n(i_b).height;
        if balance > 1 {
            let i_f = self.n(i_c).child1;
            let i_g = self.n(i_c).child2;
            self.m(i_c).child1 = i_a;
            self.m(i_c).parent = a.parent;
            self.m(i_a).parent = i_c;
            let c_parent = self.n(i_c).parent;
            if c_parent != NULL_NODE {
                if self.n(c_parent).child1 == i_a {
                    self.m(c_parent).child1 = i_c;
                } else {
                    self.m(c_parent).child2 = i_c;
                }
            } else {
                self.root = i_c;
            }
            let (keep, give) = if self.n(i_f).height > self.n(i_g).height {
                (i_f, i_g)
            } else {
                (i_g, i_f)
            };
            self.m(i_c).child2 = keep;
            self.m(i_a).child2 = give;
            self.m(give).parent = i_a;
            let a_aabb = Aabb::combine(&self.n(i_b).aabb, &self.n(give).aabb);
            self.m(i_a).aabb = a_aabb;
            let c_aabb = Aabb::combine(&a_aabb, &self.n(keep).aabb);
            self.m(i_c).aabb = c_aabb;
            let a_height = 1 + self.n(i_b).height.max(self.n(give).height);
            self.m(i_a).height = a_height;
            let c_height = 1 + a_height.max(self.n(keep).height);
            self.m(i_c).height = c_height;
            return i_c;
        }
        if balance < -1 {
            let i_d = self.n(i_b).child1;
            let i_e = self.n(i_b).child2;
            self.m(i_b).child1 = i_a;
            self.m(i_b).parent = a.parent;
            self.m(i_a).parent = i_b;
            let b_parent = self.n(i_b).parent;
            if b_parent != NULL_NODE {
                if self.n(b_parent).child1 == i_a {
                    self.m(b_parent).child1 = i_b;
                } else {
                    self.m(b_parent).child2 = i_b;
                }
            } else {
                self.root = i_b;
            }
            let (keep, give) = if self.n(i_d).height > self.n(i_e).height {
                (i_d, i_e)
            } else {
                (i_e, i_d)
            };
            self.m(i_b).child2 = keep;
            self.m(i_a).child1 = give;
            self.m(give).parent = i_a;
            let a_aabb = Aabb::combine(&self.n(i_c).aabb, &self.n(give).aabb);
            self.m(i_a).aabb = a_aabb;
            let b_aabb = Aabb::combine(&a_aabb, &self.n(keep).aabb);
            self.m(i_b).aabb = b_aabb;
            let a_height = 1 + self.n(i_c).height.max(self.n(give).height);
            self.m(i_a).height = a_height;
            let b_height = 1 + a_height.max(self.n(keep).height);
            self.m(i_b).height = b_height;
            return i_b;
        }
        i_a
    }
}
