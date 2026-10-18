// System.Random de .NET Framework 2.0 (generador sustractivo de Knuth), para que los números
// aleatorios tengan la misma distribución que en el original.

const MBIG: i32 = i32::MAX;
const MSEED: i32 = 161_803_398;

pub struct Random {
    inext: usize,
    inextp: usize,
    seed_array: [i32; 56],
}

impl Random {
    pub fn new(seed: i32) -> Self {
        let mut seed_array = [0i32; 56];
        let mut mj = MSEED.wrapping_sub(seed.wrapping_abs());
        seed_array[55] = mj;
        let mut mk = 1i32;
        for i in 1..55 {
            let ii = (21 * i) % 55;
            seed_array[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(MBIG);
            }
            mj = seed_array[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + (i + 30) % 55]);
                if seed_array[i] < 0 {
                    seed_array[i] = seed_array[i].wrapping_add(MBIG);
                }
            }
        }
        Self {
            inext: 0,
            inextp: 21,
            seed_array,
        }
    }

    /// `new Random()`: la semilla es Environment.TickCount (milisegundos desde el arranque).
    pub fn con_reloj() -> Self {
        Self::new((macroquad::miniquad::date::now() * 1000.0) as i64 as i32)
    }

    fn sample(&mut self) -> f64 {
        self.inext = if self.inext + 1 >= 56 {
            1
        } else {
            self.inext + 1
        };
        self.inextp = if self.inextp + 1 >= 56 {
            1
        } else {
            self.inextp + 1
        };
        let mut valor = self.seed_array[self.inext].wrapping_sub(self.seed_array[self.inextp]);
        if valor < 0 {
            valor = valor.wrapping_add(MBIG);
        }
        self.seed_array[self.inext] = valor;
        valor as f64 * (1.0 / MBIG as f64)
    }

    /// Random.Next(minValue, maxValue): entero en [min, max).
    pub fn next(&mut self, min: i32, max: i32) -> i32 {
        let rango = max as i64 - min as i64;
        (self.sample() * rango as f64) as i32 + min
    }

    /// Random.NextDouble().
    pub fn next_double(&mut self) -> f64 {
        self.sample()
    }
}

#[cfg(test)]
mod tests {
    use super::Random;

    #[test]
    fn misma_secuencia_que_dotnet() {
        // new Random(0).Next(0, 100) x5 en .NET Framework: 72, 81, 76, 55, 20.
        let mut r = Random::new(0);
        let v: Vec<i32> = (0..5).map(|_| r.next(0, 100)).collect();
        assert_eq!(v, [72, 81, 76, 55, 20]);
    }
}
