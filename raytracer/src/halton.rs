pub struct Halton {
    base: u32,
    index: u32,
}

impl Halton {
    fn new(base: u32) -> Self {
        Self { base, index: 1 }
    }

    #[allow(dead_code)]
    pub fn one_d(base: u32) -> impl Iterator<Item = f32> {
        let mut iter = Self::new(base);
        for _ in 0..base {
            let _ = iter.next();
        }
        iter
    }

    pub fn two_d((base1, base2): (u32, u32)) -> Halton2 {
        let mut iter = Halton2(Self::new(base1), Self::new(base2));
        for _ in 0..(base1 * base2) {
            let _ = iter.next();
        }
        iter
    }
}

impl Iterator for Halton {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // https://observablehq.com/@mythmon/halton-sequence
        let mut f = 1.0;
        let mut r = 0.0;
        let mut i = self.index as f32;
        let b = self.base as f32;
        while i > 0.0 {
            f /= b;
            r += f * (i % b);
            i = (i / b).floor();
        }
        self.index += 1;
        Some(r)
    }
}

pub struct Halton2(Halton, Halton);

impl Iterator for Halton2 {
    type Item = (<Halton as Iterator>::Item, <Halton as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.0.next(), self.1.next()) {
            (Some(h1), Some(h2)) => Some((h1, h2)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use super::Halton;

    #[test]
    fn wikipedia() {
        let mut h = Halton::new(2);
        assert_float_eq!(h.next().unwrap(), 1. / 2., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 1. / 4., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 3. / 4., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 1. / 8., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 5. / 8., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 3. / 8., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 7. / 8., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 1. / 16., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 9. / 16., abs <= 0.001);
    }

    #[test]
    fn base_three() {
        let mut h = Halton::new(3);
        assert_float_eq!(h.next().unwrap(), 1. / 3., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 2. / 3., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 1. / 9., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 4. / 9., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 7. / 9., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 2. / 9., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 5. / 9., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 8. / 9., abs <= 0.001);
    }

    #[test]
    fn one_d() {
        let mut h = Halton::one_d(2);
        assert_float_eq!(h.next().unwrap(), 3. / 4., abs <= 0.001);
        assert_float_eq!(h.next().unwrap(), 1. / 8., abs <= 0.001);
    }

    #[test]
    fn two_d() {
        let mut h = Halton::two_d((2, 3));
        assert_float_eq!(h.next().unwrap(), (7. / 8., 5. / 9.) , abs <= (0.001, 0.001));
        assert_float_eq!(h.next().unwrap(), (1. / 16., 8. / 9.) , abs <= (0.001, 0.001));
    }
}
