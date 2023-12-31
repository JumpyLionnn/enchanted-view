use crate::{key_value_match, count_tts};

key_value_match! {
    pub fn color_to_name(color: (u8, u8, u8)) -> Option<&'static str>;
    pub fn _name_to_color(name: &str) -> Option<(u8, u8, u8)>; // not used
    pub fn _get_colors() -> [(u8, u8, u8)];
    pub fn _get_color_names() -> [&'static str];
    {
        (0, 72, 186): "Absolute zero",
        (176, 191, 26): "Acid green",
        (0, 185, 232): "Aero",
        (178, 132, 190): "African violet",
        (114, 160, 193): "Air superiority blue",
        (242, 240, 230): "Alabaster",
        (240, 248, 255): "Alice blue",
        (219, 45, 67): "Alizarin",
        (196, 98, 16): "Alloy orange",
        (239, 222, 205): "Almond",
        (229, 43, 80): "Amaranth",
        (159, 43, 104): "Amaranth deep purple",
        (241, 156, 187): "Amaranth pink",
        (171, 39, 79): "Amaranth purple",
        (59, 122, 87): "Amazon",
        (255, 191, 0): "Amber",
        (153, 102, 204): "Amethyst",
        (61, 220, 132): "Android green",
        (205, 149, 117): "Antique brass",
        (102, 93, 30): "Antique bronze",
        (145, 92, 131): "Antique fuchsia",
        (132, 27, 45): "Antique ruby",
        (250, 235, 215): "Antique white",
        (251, 206, 117): "Apricot",
        (0, 255, 255): "Aqua",
        (127, 255, 212): "Aquamarine",
        (208, 255, 20): "Arctic lime",
        (75, 111, 68): "Artichoke green",
        (233, 214, 107): "Arylide yellow",
        (178, 190, 181): "Ash gray",
        (123, 160, 123): "Asparagus",
        (255, 153, 102): "Atomic tangerine",
        (253, 238, 0): "Aureolin",
        (195, 153, 83): "Aztec gold",
        (0, 127, 255): "Azure",
        (137, 207, 240): "Baby blue",
        (161, 202, 241): "Baby blue eyes",
        (244, 194, 194): "Baby pink",
        (254, 254, 250): "Baby powder",
        (255, 145, 175): "Baker-Miller pink",
        (250, 231, 181): "Banana mania",
        (224, 33, 138): "Barbie pink",
        (124, 10, 2): "Barn red",
        (132, 132, 130): "Battleship gray",
        (188, 212, 230): "Beau blue",
        (159, 129, 112): "Beaver",
        (245, 245, 220): "Beige",
        (164, 52, 130): "Berry parfait",
        (46, 88, 148): "B'dazzled blue",
        (156, 37, 66): "Big dip 'ruby",
        (232, 142, 90): "Big foot feet",
        (255, 228, 196): "Bisque",
        (61, 43, 31): "Bistre",
        (150, 113, 23): "Bistre brown",
        (202, 224, 13): "Bitter lemon",
        (254, 111, 94): "Bittersweet",
        (191, 79, 81): "Bittersweet shimmer",
        (0, 0, 0): "Black",
        (61, 12, 2): "Black bean",
        (84, 98, 111): "Black coral",
        (59, 60, 54): "Black olive",
        (191, 175, 178): "Black shadows",
        (255, 235, 205): "Blanched almond",
        (165, 113, 100): "Blast-off bronze",
        (49, 140, 231): "Bleu de France",
        (80, 191, 230): "Blizzard blue",
        (102, 0, 0): "Blood red",
        (0, 0, 255): "Blue",
        (162, 162, 208): "Blue bell",
        (102, 153, 204): "Blue-gray",
        (13, 152, 186): "Blue-green",
        (93, 173, 236): "Blue jeans",
        (11, 16, 162): "Blue ribbon",
        (18, 97, 128): "Blue sapphire",
        (138, 43, 226): "Blue-violet",
        (80, 114, 167): "Blue yonder",
        (79, 134, 247): "Blueberry",
        (60, 105, 231): "Bluetiful",
        (222, 93, 131): "Blush",
        (121, 68, 59): "Bole",
        (227, 218, 201): "Bone",
        (221, 226, 106): "Booger buster",
        (203, 65, 84): "Brick red",
        (102, 255, 0): "Bright green",
        (216, 145, 239): "Bright lilac",
        (195, 33, 72): "Bright maroon",
        (25, 116, 210): "Bright navy blue",
        (255, 0, 127): "Bright pink",
        (8, 232, 222): "Bright turquoise",
        (230, 103, 206): "Brilliant rose",
        (251, 96, 127): "Brink pink",
        (0, 66, 37): "British racing green",
        (205, 127, 50): "Bronze",
        (150, 75, 0): "Brown",
        (175, 110, 77): "Brown sugar",
        (123, 182, 97): "Bud green",
        (240, 220, 130): "Buff",
        (128, 0, 32): "Burgundy",
        (222, 184, 135): "Burlywood",
        (161, 122, 116): "Burnished brown",
        (204, 85, 0): "Burnt orange",
        (233, 116, 81): "Burnt sienna",
        (138, 51, 36): "Burnt umber",
        (189, 51, 164): "Byzantine",
        (112, 41, 99): "Byzantium",
        (83, 104, 114): "Cadet",
        (95, 158, 160): "Cadet blue",
        (145, 163, 176): "Cadet gray",
        (0, 107, 60): "Cadmium",
        (237, 135, 45): "Cadmium orange",
        (227, 0, 34): "Cadmium red",
        (255, 246, 0): "Cadmium yellow",
        (166, 123, 91): "Café au lait",
        (75, 54, 33): "Café Noir",
        (163, 193, 173): "Cambridge blue",
        (193, 154, 107): "Camel",
        (239, 187, 204): "Cameo pink",
        (255, 255, 153): "Canary",
        (255, 239, 0): "Canary yellow",
        (255, 8, 0): "Candy apple red",
        (228, 113, 122): "Candy pink",
        (196, 30, 58): "Cardinal",
        (0, 204, 153): "Caribbean green",
        (150, 0, 24): "Carmine",
        (255, 166, 201): "Carnation pink",
        (86, 160, 211): "Carolina blue",
        (237, 145, 33): "Carrot orange",
        (112, 54, 66): "Catawba",
        (202, 52, 53): "Cedar chest",
        (172, 225, 175): "Celadon",
        (178, 255, 255): "Celeste",
        (222, 49, 99): "Cerise",
        (0, 123, 167): "Cerulean",
        (42, 82, 190): "Cerulean blue",
        (109, 155, 195): "Cerulean frost",
        (247, 231, 206): "Champagne",
        (241, 221, 207): "Champagne pink",
        (54, 69, 79): "Charcoal",
        (35, 43, 43): "Charleston green",
        (230, 143, 172): "Charm pink",
        (127, 255, 0): "Chartreuse",
        (255, 183, 197): "Cherry blossom pink",
        (149, 69, 53): "Chestnut",
        (222, 111, 161): "China pink",
        (170, 56, 30): "Chinese red",
        (133, 96, 126): "Chinese violet",
        (123, 63, 0): "Chocolate",
        (152, 129, 123): "Cinereous",
        (205, 96, 126): "Cinnamon Satin",
        (228, 208, 10): "Citrine",
        (158, 169, 31): "Citron",
        (127, 23, 52): "Claret",
        (0, 71, 171): "Cobalt blue",
        (210, 105, 30): "Cocoa brown",
        (150, 90, 62): "Coconut",
        (111, 78, 55): "Coffee",
        (196, 216, 226): "Columbia blue",
        (248, 131, 121): "Congo pink",
        (140, 146, 172): "Cool grey",
        (184, 115, 51): "Copper",
        (173, 111, 105): "Copper penny",
        (203, 109, 81): "Copper red",
        (153, 102, 102): "Copper rose",
        (255, 56, 0): "Coquelicot",
        (255, 127, 80): "Coral",
        (248, 131, 121): "Coral pink",
        (137, 63, 69): "Cordovan",
        (100, 149, 237): "Cornflower blue",
        (255, 248, 220): "Cornsilk",
        (46, 45, 136): "Cosmic cobalt",
        (255, 248, 231): "Cosmic latte",
        (129, 97, 60): "Coyote brown",
        (255, 188, 217): "Cotton candy",
        (255, 253, 208): "Cream",
        (220, 20, 60): "Crimson",
        (245, 245, 245): "Cultured",
        (0, 255, 255): "Cyan",
        (88, 66, 124): "Cyber grape",
        (255, 211, 0): "Cyber yellow",
        (245, 111, 161): "Cyclamen"
        // TODO: Add more colors from https://en.wikipedia.org/wiki/List_of_colors_(alphabetical)
    }
}