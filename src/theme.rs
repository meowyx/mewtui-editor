use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border_active: Color,
    pub border_inactive: Color,
    pub syntax_keyword: Color,
    pub syntax_string: Color,
    pub syntax_comment: Color,
    pub syntax_function: Color,
}

pub const ALL_THEMES: [Theme; 20] = [
    // ========================================================
    //  DEFAULT — Dracula
    // ========================================================
    Theme {
        name: "dracula",
        bg: Color::Rgb(40, 42, 54),
        fg: Color::Rgb(248, 248, 242),
        accent: Color::Rgb(189, 147, 249),
        border_active: Color::Rgb(189, 147, 249),
        border_inactive: Color::Rgb(98, 114, 164),
        syntax_keyword: Color::Rgb(255, 121, 198),
        syntax_string: Color::Rgb(241, 250, 140),
        syntax_comment: Color::Rgb(98, 114, 164),
        syntax_function: Color::Rgb(80, 250, 123),
    },

    // ========================================================
    //  Modern dark themes
    // ========================================================

    // Catppuccin Mocha — pastel dark, cozy
    Theme {
        name: "catppuccin",
        bg: Color::Rgb(30, 30, 46),
        fg: Color::Rgb(205, 214, 244),
        accent: Color::Rgb(137, 180, 250),
        border_active: Color::Rgb(137, 180, 250),
        border_inactive: Color::Rgb(69, 71, 90),
        syntax_keyword: Color::Rgb(203, 166, 247),
        syntax_string: Color::Rgb(166, 227, 161),
        syntax_comment: Color::Rgb(108, 112, 134),
        syntax_function: Color::Rgb(137, 180, 250),
    },
    // Tokyo Night — purple haze, calm city lights
    Theme {
        name: "tokyo-night",
        bg: Color::Rgb(26, 27, 38),
        fg: Color::Rgb(169, 177, 214),
        accent: Color::Rgb(122, 162, 247),
        border_active: Color::Rgb(122, 162, 247),
        border_inactive: Color::Rgb(54, 58, 79),
        syntax_keyword: Color::Rgb(187, 154, 247),
        syntax_string: Color::Rgb(158, 206, 106),
        syntax_comment: Color::Rgb(86, 95, 137),
        syntax_function: Color::Rgb(122, 162, 247),
    },
    // Rosé Pine — muted rose and gold, elegant
    Theme {
        name: "rose-pine",
        bg: Color::Rgb(25, 23, 36),
        fg: Color::Rgb(224, 222, 244),
        accent: Color::Rgb(235, 188, 186),
        border_active: Color::Rgb(235, 188, 186),
        border_inactive: Color::Rgb(110, 106, 134),
        syntax_keyword: Color::Rgb(49, 116, 143),
        syntax_string: Color::Rgb(246, 193, 119),
        syntax_comment: Color::Rgb(110, 106, 134),
        syntax_function: Color::Rgb(235, 188, 186),
    },
    // Synthwave '84 — neon retro-future
    Theme {
        name: "synthwave",
        bg: Color::Rgb(34, 35, 60),
        fg: Color::Rgb(255, 230, 255),
        accent: Color::Rgb(255, 56, 187),
        border_active: Color::Rgb(255, 56, 187),
        border_inactive: Color::Rgb(73, 69, 124),
        syntax_keyword: Color::Rgb(255, 117, 206),
        syntax_string: Color::Rgb(255, 231, 107),
        syntax_comment: Color::Rgb(104, 97, 168),
        syntax_function: Color::Rgb(54, 243, 255),
    },
    // Night Owl — rich blues and greens
    Theme {
        name: "night-owl",
        bg: Color::Rgb(1, 22, 39),
        fg: Color::Rgb(214, 222, 235),
        accent: Color::Rgb(130, 170, 255),
        border_active: Color::Rgb(130, 170, 255),
        border_inactive: Color::Rgb(68, 85, 106),
        syntax_keyword: Color::Rgb(199, 146, 234),
        syntax_string: Color::Rgb(173, 219, 103),
        syntax_comment: Color::Rgb(99, 119, 142),
        syntax_function: Color::Rgb(130, 170, 255),
    },
    // Kanagawa — inspired by the Great Wave, warm ink
    Theme {
        name: "kanagawa",
        bg: Color::Rgb(27, 27, 35),
        fg: Color::Rgb(220, 215, 186),
        accent: Color::Rgb(127, 180, 202),
        border_active: Color::Rgb(127, 180, 202),
        border_inactive: Color::Rgb(84, 84, 109),
        syntax_keyword: Color::Rgb(149, 127, 184),
        syntax_string: Color::Rgb(152, 187, 108),
        syntax_comment: Color::Rgb(114, 113, 105),
        syntax_function: Color::Rgb(127, 180, 202),
    },
    // Ayu Dark — clean, vibrant accents on deep dark
    Theme {
        name: "ayu-dark",
        bg: Color::Rgb(10, 14, 20),
        fg: Color::Rgb(179, 186, 197),
        accent: Color::Rgb(255, 180, 84),
        border_active: Color::Rgb(255, 180, 84),
        border_inactive: Color::Rgb(60, 68, 81),
        syntax_keyword: Color::Rgb(255, 139, 38),
        syntax_string: Color::Rgb(170, 217, 76),
        syntax_comment: Color::Rgb(92, 101, 117),
        syntax_function: Color::Rgb(255, 180, 84),
    },
    // Everforest Dark — nature greens, gentle on the eyes
    Theme {
        name: "everforest",
        bg: Color::Rgb(39, 52, 46),
        fg: Color::Rgb(211, 198, 170),
        accent: Color::Rgb(163, 190, 140),
        border_active: Color::Rgb(163, 190, 140),
        border_inactive: Color::Rgb(79, 98, 87),
        syntax_keyword: Color::Rgb(230, 126, 128),
        syntax_string: Color::Rgb(163, 190, 140),
        syntax_comment: Color::Rgb(135, 144, 128),
        syntax_function: Color::Rgb(125, 196, 170),
    },
    // One Dark — Atom / VS Code classic
    Theme {
        name: "one-dark",
        bg: Color::Rgb(40, 44, 52),
        fg: Color::Rgb(171, 178, 191),
        accent: Color::Rgb(97, 175, 239),
        border_active: Color::Rgb(97, 175, 239),
        border_inactive: Color::Rgb(76, 82, 99),
        syntax_keyword: Color::Rgb(198, 120, 221),
        syntax_string: Color::Rgb(152, 195, 121),
        syntax_comment: Color::Rgb(92, 99, 112),
        syntax_function: Color::Rgb(97, 175, 239),
    },
    // Monokai — sublime text classic
    Theme {
        name: "monokai",
        bg: Color::Rgb(39, 40, 34),
        fg: Color::Rgb(248, 248, 242),
        accent: Color::Rgb(166, 226, 46),
        border_active: Color::Rgb(166, 226, 46),
        border_inactive: Color::Rgb(117, 113, 94),
        syntax_keyword: Color::Rgb(249, 38, 114),
        syntax_string: Color::Rgb(230, 219, 116),
        syntax_comment: Color::Rgb(117, 113, 94),
        syntax_function: Color::Rgb(166, 226, 46),
    },
    // GitHub Dark — clean modern dark
    Theme {
        name: "github-dark",
        bg: Color::Rgb(13, 17, 23),
        fg: Color::Rgb(230, 237, 243),
        accent: Color::Rgb(47, 129, 247),
        border_active: Color::Rgb(47, 129, 247),
        border_inactive: Color::Rgb(48, 54, 61),
        syntax_keyword: Color::Rgb(255, 123, 114),
        syntax_string: Color::Rgb(165, 214, 255),
        syntax_comment: Color::Rgb(139, 148, 158),
        syntax_function: Color::Rgb(210, 168, 255),
    },
    // Gruvbox Dark — warm retro browns
    Theme {
        name: "gruvbox",
        bg: Color::Rgb(40, 40, 40),
        fg: Color::Rgb(235, 219, 178),
        accent: Color::Rgb(250, 189, 47),
        border_active: Color::Rgb(250, 189, 47),
        border_inactive: Color::Rgb(80, 73, 69),
        syntax_keyword: Color::Rgb(251, 73, 52),
        syntax_string: Color::Rgb(184, 187, 38),
        syntax_comment: Color::Rgb(146, 131, 116),
        syntax_function: Color::Rgb(131, 165, 152),
    },
    // Nord — arctic blue, calm
    Theme {
        name: "nord",
        bg: Color::Rgb(46, 52, 64),
        fg: Color::Rgb(216, 222, 233),
        accent: Color::Rgb(136, 192, 208),
        border_active: Color::Rgb(136, 192, 208),
        border_inactive: Color::Rgb(67, 76, 94),
        syntax_keyword: Color::Rgb(129, 161, 193),
        syntax_string: Color::Rgb(163, 190, 140),
        syntax_comment: Color::Rgb(76, 86, 106),
        syntax_function: Color::Rgb(136, 192, 208),
    },
    // Solarized Dark
    Theme {
        name: "solarized",
        bg: Color::Rgb(0, 43, 54),
        fg: Color::Rgb(131, 148, 150),
        accent: Color::Rgb(38, 139, 210),
        border_active: Color::Rgb(38, 139, 210),
        border_inactive: Color::Rgb(0, 63, 74),
        syntax_keyword: Color::Rgb(133, 153, 0),
        syntax_string: Color::Rgb(42, 161, 152),
        syntax_comment: Color::Rgb(88, 110, 117),
        syntax_function: Color::Rgb(38, 139, 210),
    },

    // ========================================================
    //  Retro themes
    // ========================================================

    // phosphor — 90s hacker, The Matrix
    Theme {
        name: "phosphor",
        bg: Color::Rgb(0, 10, 0),
        fg: Color::Rgb(0, 255, 65),
        accent: Color::Rgb(0, 153, 40),
        border_active: Color::Rgb(0, 255, 65),
        border_inactive: Color::Rgb(0, 153, 40),
        syntax_keyword: Color::Rgb(0, 255, 130),
        syntax_string: Color::Rgb(100, 255, 100),
        syntax_comment: Color::Rgb(0, 100, 30),
        syntax_function: Color::Rgb(150, 255, 150),
    },
    // amber — old amber monitor
    Theme {
        name: "amber",
        bg: Color::Rgb(10, 5, 0),
        fg: Color::Rgb(255, 176, 0),
        accent: Color::Rgb(153, 102, 0),
        border_active: Color::Rgb(255, 176, 0),
        border_inactive: Color::Rgb(153, 102, 0),
        syntax_keyword: Color::Rgb(255, 200, 50),
        syntax_string: Color::Rgb(255, 220, 100),
        syntax_comment: Color::Rgb(120, 80, 0),
        syntax_function: Color::Rgb(255, 230, 150),
    },
    // cobalt — blue phosphor CRT
    Theme {
        name: "cobalt",
        bg: Color::Rgb(0, 5, 25),
        fg: Color::Rgb(0, 200, 255),
        accent: Color::Rgb(0, 80, 140),
        border_active: Color::Rgb(0, 200, 255),
        border_inactive: Color::Rgb(0, 80, 140),
        syntax_keyword: Color::Rgb(100, 200, 255),
        syntax_string: Color::Rgb(150, 220, 255),
        syntax_comment: Color::Rgb(0, 60, 100),
        syntax_function: Color::Rgb(180, 230, 255),
    },
    // terminal — plain white on black
    Theme {
        name: "terminal",
        bg: Color::Rgb(0, 0, 0),
        fg: Color::Rgb(204, 204, 204),
        accent: Color::Rgb(102, 102, 102),
        border_active: Color::Rgb(204, 204, 204),
        border_inactive: Color::Rgb(102, 102, 102),
        syntax_keyword: Color::Rgb(200, 200, 255),
        syntax_string: Color::Rgb(200, 255, 200),
        syntax_comment: Color::Rgb(128, 128, 128),
        syntax_function: Color::Rgb(255, 255, 200),
    },
    // void — brutalist high contrast
    Theme {
        name: "void",
        bg: Color::Rgb(0, 0, 0),
        fg: Color::Rgb(255, 255, 255),
        accent: Color::Rgb(255, 0, 0),
        border_active: Color::Rgb(255, 255, 255),
        border_inactive: Color::Rgb(80, 80, 80),
        syntax_keyword: Color::Rgb(255, 255, 255),
        syntax_string: Color::Rgb(200, 200, 200),
        syntax_comment: Color::Rgb(100, 100, 100),
        syntax_function: Color::Rgb(255, 255, 255),
    },
];
