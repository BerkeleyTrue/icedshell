#[derive(Debug, Clone)]
pub struct SpacingScale {
    values: [f32; 10],
}

impl SpacingScale {
    pub const DEFAULT: Self = Self {
        values: [2.0, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0],
    };

    /// 2px
    #[inline(always)]
    #[must_use]
    pub const fn xxs(&self) -> f32 {
        self.values[0]
    }

    /// 4px
    #[inline(always)]
    #[must_use]
    pub const fn xs(&self) -> f32 {
        self.values[1]
    }

    /// 8px
    #[inline(always)]
    #[must_use]
    pub const fn sm(&self) -> f32 {
        self.values[2]
    }

    /// 12px
    #[inline(always)]
    #[must_use]
    pub const fn md(&self) -> f32 {
        self.values[3]
    }

    /// 16px
    #[inline(always)]
    #[must_use]
    pub const fn lg(&self) -> f32 {
        self.values[4]
    }

    /// 24px
    #[inline(always)]
    #[must_use]
    pub const fn xl(&self) -> f32 {
        self.values[5]
    }

    /// 32px
    #[inline(always)]
    #[must_use]
    pub const fn xl2(&self) -> f32 {
        self.values[6]
    }

    /// 48px
    #[inline(always)]
    #[must_use]
    pub const fn xl3(&self) -> f32 {
        self.values[7]
    }

    /// 64px
    #[inline(always)]
    #[must_use]
    pub const fn xl4(&self) -> f32 {
        self.values[8]
    }

    /// 96px
    #[inline(always)]
    #[must_use]
    pub const fn xl5(&self) -> f32 {
        self.values[9]
    }
}
