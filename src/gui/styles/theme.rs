use iced::Color;

pub struct Theme {
    pub urgency_expired: Color,
    pub urgency_today: Color,
    pub urgency_soon: Color,
    pub urgency_later: Color,
    
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            urgency_expired: Color::from_rgb(0.906, 0.298, 0.235),
            urgency_today: Color::from_rgb(0.953, 0.612, 0.071),
            urgency_soon: Color::from_rgb(0.153, 0.682, 0.376),
            urgency_later: Color::from_rgb(0.584, 0.647, 0.651),
            
            primary: Color::from_rgb(0.204, 0.596, 0.859),
            secondary: Color::from_rgb(0.180, 0.800, 0.443),
            background: Color::from_rgb(1.0, 1.0, 1.0),
            surface: Color::from_rgb(0.961, 0.961, 0.961),
            text: Color::from_rgb(0.173, 0.243, 0.314),
            text_secondary: Color::from_rgb(0.4, 0.4, 0.4),
            
            success: Color::from_rgb(0.153, 0.682, 0.376),
            warning: Color::from_rgb(0.953, 0.612, 0.071),
            error: Color::from_rgb(0.906, 0.298, 0.235),
        }
    }
}

impl Theme {
    pub fn urgency_color(urgency: u32) -> Color {
        match urgency {
            3 => Self::default().urgency_expired,
            2 => Self::default().urgency_today,
            1 => Self::default().urgency_soon,
            _ => Self::default().urgency_later,
        }
    }
}
