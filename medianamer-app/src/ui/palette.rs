use iced::Color;

pub struct Palette {
    pub bg:         Color,
    pub surface:    Color,
    pub surface2:   Color,
    pub border:     Color,
    pub text:       Color,
    pub text2:      Color,
    pub text3:      Color,
    pub accent:     Color,
    pub accent_bg:  Color,
    pub success:    Color,
    pub success_bg: Color,
    pub warn:       Color,
    pub warn_bg:    Color,
    pub danger:     Color,
    pub danger_bg:  Color,
}

pub fn palette(is_dark: bool) -> Palette {
    if is_dark {
        Palette {
            bg:         Color::from_rgb8(26,  25,  23),
            surface:    Color::from_rgb8(35,  34,  32),
            surface2:   Color::from_rgb8(42,  40,  38),
            border:     Color::from_rgb8(58,  56,  53),
            text:       Color::from_rgb8(240, 237, 232),
            text2:      Color::from_rgb8(154, 149, 144),
            text3:      Color::from_rgb8(106, 101, 96),
            accent:     Color::from_rgb8(91,  138, 228),
            accent_bg:  Color::from_rgb8(30,  42,  69),
            success:    Color::from_rgb8(58,  175, 107),
            success_bg: Color::from_rgb8(14,  42,  28),
            warn:       Color::from_rgb8(212, 122, 32),
            warn_bg:    Color::from_rgb8(42,  30,  10),
            danger:     Color::from_rgb8(224, 92,  76),
            danger_bg:  Color::from_rgb8(58,  26,  24),
        }
    } else {
        Palette {
            bg:         Color::from_rgb8(244, 243, 241),
            surface:    Color::from_rgb8(255, 255, 255),
            surface2:   Color::from_rgb8(240, 238, 236),
            border:     Color::from_rgb8(216, 212, 207),
            text:       Color::from_rgb8(28,  25,  23),
            text2:      Color::from_rgb8(107, 101, 96),
            text3:      Color::from_rgb8(156, 150, 144),
            accent:     Color::from_rgb8(59,  111, 212),
            accent_bg:  Color::from_rgb8(232, 237, 248),
            success:    Color::from_rgb8(26,  127, 75),
            success_bg: Color::from_rgb8(230, 244, 236),
            warn:       Color::from_rgb8(180, 83,  9),
            warn_bg:    Color::from_rgb8(254, 243, 226),
            danger:     Color::from_rgb8(192, 57,  43),
            danger_bg:  Color::from_rgb8(253, 236, 234),
        }
    }
}
