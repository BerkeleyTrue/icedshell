#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadiusScale {
    values: [f32; 8],
}

impl RadiusScale {
    pub const DEFAULT: Self = Self {
        values: [0.0, 2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 9999.0],
    };

    #[inline(always)]
    #[must_use]
    pub const fn none(&self) -> f32 {
        self.values[0]
    }

    /// 2px
    #[inline(always)]
    #[must_use]
    pub const fn xs(&self) -> f32 {
        self.values[1]
    }

    /// 4px
    #[inline(always)]
    #[must_use]
    pub const fn sm(&self) -> f32 {
        self.values[2]
    }

    /// 6px
    #[inline(always)]
    #[must_use]
    pub const fn md(&self) -> f32 {
        self.values[3]
    }

    /// 8px
    #[inline(always)]
    #[must_use]
    pub const fn lg(&self) -> f32 {
        self.values[4]
    }

    ///  12px
    #[inline(always)]
    #[must_use]
    pub const fn xl(&self) -> f32 {
        self.values[5]
    }

    /// 16px
    #[inline(always)]
    #[must_use]
    pub const fn xl2(&self) -> f32 {
        self.values[6]
    }

    /// full
    #[inline(always)]
    #[must_use]
    pub const fn full(&self) -> f32 {
        self.values[7]
    }
}
