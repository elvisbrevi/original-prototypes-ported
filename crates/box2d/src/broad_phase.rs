// b2BroadPhase: los proxies que se movieron buscan con quién solaparse en el árbol; las parejas
// se ordenan por identificador y se quitan las repetidas.

use crate::collision::{Aabb, test_overlap_aabb};
use crate::dynamic_tree::DynamicTree;
use crate::math::Vec2;

pub const NULL_PROXY: i32 = -1;

#[derive(Default)]
pub struct BroadPhase {
    tree: DynamicTree,
    proxy_count: i32,
    move_buffer: Vec<i32>,
}

impl BroadPhase {
    pub fn create_proxy(&mut self, aabb: &Aabb, user_data: usize) -> i32 {
        let proxy_id = self.tree.create_proxy(aabb, user_data);
        self.proxy_count += 1;
        self.buffer_move(proxy_id);
        proxy_id
    }

    pub fn destroy_proxy(&mut self, proxy_id: i32) {
        self.un_buffer_move(proxy_id);
        self.proxy_count -= 1;
        self.tree.destroy_proxy(proxy_id);
    }

    pub fn move_proxy(&mut self, proxy_id: i32, aabb: &Aabb, displacement: Vec2) {
        if self.tree.move_proxy(proxy_id, aabb, displacement) {
            self.buffer_move(proxy_id);
        }
    }

    pub fn touch_proxy(&mut self, proxy_id: i32) {
        self.buffer_move(proxy_id);
    }

    fn buffer_move(&mut self, proxy_id: i32) {
        self.move_buffer.push(proxy_id);
    }

    fn un_buffer_move(&mut self, proxy_id: i32) {
        for m in &mut self.move_buffer {
            if *m == proxy_id {
                *m = NULL_PROXY;
            }
        }
    }

    pub fn get_user_data(&self, proxy_id: i32) -> usize {
        self.tree.get_user_data(proxy_id)
    }

    pub fn test_overlap(&self, proxy_id_a: i32, proxy_id_b: i32) -> bool {
        test_overlap_aabb(
            self.tree.get_fat_aabb(proxy_id_a),
            self.tree.get_fat_aabb(proxy_id_b),
        )
    }

    /// UpdatePairs: las parejas nuevas (sus datos de usuario), en el orden en que el original
    /// llama a AddPair.
    pub fn update_pairs(&mut self) -> Vec<(usize, usize)> {
        let mut pairs: Vec<(i32, i32)> = Vec::new();
        for &query_proxy_id in &self.move_buffer {
            if query_proxy_id == NULL_PROXY {
                continue;
            }
            let fat_aabb = *self.tree.get_fat_aabb(query_proxy_id);
            self.tree.query(&fat_aabb, |proxy_id| {
                if proxy_id != query_proxy_id {
                    pairs.push((proxy_id.min(query_proxy_id), proxy_id.max(query_proxy_id)));
                }
                true
            });
        }
        self.move_buffer.clear();
        pairs.sort_unstable();
        pairs.dedup();
        pairs
            .into_iter()
            .map(|(a, b)| (self.tree.get_user_data(a), self.tree.get_user_data(b)))
            .collect()
    }

    pub fn get_fat_aabb(&self, proxy_id: i32) -> &Aabb {
        self.tree.get_fat_aabb(proxy_id)
    }
}
