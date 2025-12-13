pub struct Border {
    pub top: &'static str,
    pub bottom: &'static str,
    pub left: &'static str,
    pub right: &'static str,
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
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
