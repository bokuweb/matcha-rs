/// Border characters used by components such as [`crate::borderize::Borderize`].
///
/// This is a simplified border definition (corners + sides).
pub struct Border {
    /// Horizontal top line character.
    pub top: &'static str,
    /// Horizontal bottom line character.
    pub bottom: &'static str,
    /// Vertical left line character.
    pub left: &'static str,
    /// Vertical right line character.
    pub right: &'static str,
    /// Top-left corner character.
    pub top_left: &'static str,
    /// Top-right corner character.
    pub top_right: &'static str,
    /// Bottom-left corner character.
    pub bottom_left: &'static str,
    /// Bottom-right corner character.
    pub bottom_right: &'static str,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            top: "─",
            bottom: "─",
            left: "│",
            right: "│",
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
        }
    }
}

/*
          Top:          "─",
        Bottom:       "─",
        Left:         "│",
        Right:        "│",
        TopLeft:      "┌",
        TopRight:     "┐",
        BottomLeft:   "└",
        BottomRight:  "┘",
        MiddleLeft:   "├",
        MiddleRight:  "┤",
        Middle:       "┼",
        MiddleTop:    "┬",
        MiddleBottom: "┴",
    }

    roundedBorder = Border{
        Top:          "─",
        Bottom:       "─",
        Left:         "│",
        Right:        "│",
        TopLeft:      "╭",
        TopRight:     "╮",
        BottomLeft:   "╰",
        BottomRight:  "╯",
        MiddleLeft:   "├",
        MiddleRight:  "┤",
        Middle:       "┼",
        MiddleTop:    "┬",
        MiddleBottom: "┴",
    }

    blockBorder = Border{
        Top:         "█",
        Bottom:      "█",
        Left:        "█",
        Right:       "█",
        TopLeft:     "█",
        TopRight:    "█",
        BottomLeft:  "█",
        BottomRight: "█",
    }

    outerHalfBlockBorder = Border{
        Top:         "▀",
        Bottom:      "▄",
        Left:        "▌",
        Right:       "▐",
        TopLeft:     "▛",
        TopRight:    "▜",
        BottomLeft:  "▙",
        BottomRight: "▟",
    }

    innerHalfBlockBorder = Border{
        Top:         "▄",
        Bottom:      "▀",
        Left:        "▐",
        Right:       "▌",
        TopLeft:     "▗",
        TopRight:    "▖",
        BottomLeft:  "▝",
        BottomRight: "▘",
    }

    thickBorder = Border{
        Top:          "━",
        Bottom:       "━",
        Left:         "┃",
        Right:        "┃",
        TopLeft:      "┏",
        TopRight:     "┓",
        BottomLeft:   "┗",
        BottomRight:  "┛",
        MiddleLeft:   "┣",
        MiddleRight:  "┫",
        Middle:       "╋",
        MiddleTop:    "┳",
        MiddleBottom: "┻",
    }

    doubleBorder = Border{
        Top:          "═",
        Bottom:       "═",
        Left:         "║",
        Right:        "║",
        TopLeft:      "╔",
        TopRight:     "╗",
        BottomLeft:   "╚",
        BottomRight:  "╝",
        MiddleLeft:   "╠",
        MiddleRight:  "╣",
        Middle:       "╬",
        MiddleTop:    "╦",
        MiddleBottom: "╩",
    }

    hiddenBorder = Border{
        Top:          " ",
        Bottom:       " ",
        Left:         " ",
        Right:        " ",
        TopLeft:      " ",
        TopRight:     " ",
        BottomLeft:   " ",
        BottomRight:  " ",
        MiddleLeft:   " ",
        MiddleRight:  " ",
        Middle:       " ",
        MiddleTop:    " ",
        MiddleBottom: " ",
    }
)
*/
