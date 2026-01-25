#!/usr/bin/env python3
"""Generate pixel art sprites for the RTS game."""

from PIL import Image, ImageDraw
import os

# Output directory
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "textures", "sprites")
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Faction colors (base colors, units will be tinted)
CONTINUITY_BLUE = (51, 102, 204)
COLLEGIUM_GOLD = (204, 153, 51)


def create_infantry_sprite(size=32):
    """Create a soldier/infantry sprite - humanoid with rifle."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx, cy = size // 2, size // 2
    
    # Head (circle at top)
    head_r = size // 8
    draw.ellipse([cx - head_r, 4, cx + head_r, 4 + head_r * 2], fill=(200, 200, 200, 255))
    
    # Body (torso)
    body_top = 4 + head_r * 2
    body_bottom = cy + size // 4
    draw.rectangle([cx - size//6, body_top, cx + size//6, body_bottom], fill=(100, 100, 100, 255))
    
    # Legs
    leg_w = size // 10
    draw.rectangle([cx - size//5, body_bottom, cx - size//5 + leg_w, size - 4], fill=(80, 80, 80, 255))
    draw.rectangle([cx + size//5 - leg_w, body_bottom, cx + size//5, size - 4], fill=(80, 80, 80, 255))
    
    # Arms
    draw.rectangle([cx - size//4, body_top + 2, cx - size//6, body_bottom - 4], fill=(100, 100, 100, 255))
    draw.rectangle([cx + size//6, body_top + 2, cx + size//4, body_bottom - 4], fill=(100, 100, 100, 255))
    
    # Rifle (diagonal line from hands)
    draw.line([cx + size//4, body_top + 4, cx + size//3 + 4, body_top - 4], fill=(60, 60, 60, 255), width=2)
    
    return img


def create_ranger_sprite(size=32):
    """Create a ranger/sniper sprite - taller, with long rifle."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx = size // 2
    
    # Slimmer, taller proportions
    # Head with hood
    head_r = size // 10
    draw.ellipse([cx - head_r - 1, 2, cx + head_r + 1, 2 + head_r * 2 + 2], fill=(60, 80, 60, 255))  # Hood
    draw.ellipse([cx - head_r, 3, cx + head_r, 3 + head_r * 2], fill=(180, 160, 140, 255))  # Face
    
    # Cloak/body (triangular)
    body_top = 3 + head_r * 2
    draw.polygon([
        (cx, body_top),
        (cx - size//4, size - 6),
        (cx + size//4, size - 6)
    ], fill=(50, 70, 50, 255))
    
    # Long sniper rifle
    draw.line([cx - 2, body_top + 4, cx + size//2 - 2, 0], fill=(40, 40, 40, 255), width=3)
    
    # Scope glint
    draw.ellipse([cx + size//4 - 1, 2, cx + size//4 + 2, 5], fill=(100, 200, 255, 255))
    
    return img


def create_harvester_sprite(size=36):
    """Create a harvester/worker vehicle sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Main body (boxy vehicle)
    body_margin = 4
    draw.rectangle([body_margin, size//3, size - body_margin, size - 6], fill=(180, 140, 60, 255))
    
    # Cab (smaller box on top)
    cab_w = size // 3
    cx = size // 2
    draw.rectangle([cx - cab_w//2, 6, cx + cab_w//2, size//3 + 2], fill=(160, 120, 40, 255))
    
    # Window
    draw.rectangle([cx - cab_w//4, 8, cx + cab_w//4, size//4], fill=(100, 180, 220, 255))
    
    # Scoop/bucket at front
    draw.polygon([
        (2, size - 8),
        (body_margin, size//2),
        (body_margin + size//4, size//2),
        (body_margin + size//4 + 2, size - 8)
    ], fill=(120, 100, 40, 255))
    
    # Wheels/tracks
    track_y = size - 5
    draw.ellipse([body_margin, track_y - 3, body_margin + 8, track_y + 3], fill=(40, 40, 40, 255))
    draw.ellipse([size - body_margin - 8, track_y - 3, size - body_margin, track_y + 3], fill=(40, 40, 40, 255))
    
    # Cargo indicator lines
    for i in range(3):
        y = size//2 + 4 + i * 4
        draw.line([body_margin + 4, y, size - body_margin - 4, y], fill=(140, 100, 40, 255), width=1)
    
    return img


def create_depot_sprite(size=64):
    """Create a main depot/base building sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    margin = 4
    
    # Main structure (large building)
    draw.rectangle([margin, size//4, size - margin, size - margin], fill=(100, 110, 130, 255))
    
    # Roof (darker)
    draw.polygon([
        (margin, size//4),
        (size//2, margin),
        (size - margin, size//4)
    ], fill=(70, 80, 100, 255))
    
    # Windows (grid)
    win_color = (80, 160, 200, 255)
    win_size = 6
    for row in range(2):
        for col in range(4):
            wx = margin + 8 + col * 12
            wy = size//4 + 8 + row * 14
            draw.rectangle([wx, wy, wx + win_size, wy + win_size], fill=win_color)
    
    # Door
    door_w = 12
    door_h = 16
    cx = size // 2
    draw.rectangle([cx - door_w//2, size - margin - door_h, cx + door_w//2, size - margin], fill=(60, 50, 40, 255))
    
    # Antenna/comm tower
    draw.line([size - margin - 8, margin + 4, size - margin - 8, size//4], fill=(80, 80, 80, 255), width=2)
    draw.ellipse([size - margin - 11, margin + 1, size - margin - 5, margin + 7], fill=(255, 100, 100, 255))
    
    return img


def create_barracks_sprite(size=48):
    """Create a barracks building sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    margin = 3
    
    # Main building (military style, flat roof)
    draw.rectangle([margin, size//5, size - margin, size - margin], fill=(90, 85, 80, 255))
    
    # Flat reinforced roof
    draw.rectangle([margin - 1, size//5 - 3, size - margin + 1, size//5 + 2], fill=(70, 65, 60, 255))
    
    # Windows (narrow slits - bunker style)
    for i in range(3):
        wx = margin + 8 + i * 12
        draw.rectangle([wx, size//3, wx + 8, size//3 + 4], fill=(40, 60, 80, 255))
    
    # Large door (hangar style)
    door_w = 20
    cx = size // 2
    draw.rectangle([cx - door_w//2, size//2, cx + door_w//2, size - margin], fill=(50, 50, 50, 255))
    # Door lines
    draw.line([cx, size//2, cx, size - margin], fill=(40, 40, 40, 255), width=1)
    
    # Military emblem (star)
    star_cx = size - margin - 10
    star_cy = size//3 + 8
    draw.polygon([
        (star_cx, star_cy - 5),
        (star_cx + 2, star_cy - 1),
        (star_cx + 5, star_cy),
        (star_cx + 2, star_cy + 1),
        (star_cx, star_cy + 5),
        (star_cx - 2, star_cy + 1),
        (star_cx - 5, star_cy),
        (star_cx - 2, star_cy - 1),
    ], fill=(180, 50, 50, 255))
    
    return img


def create_supply_depot_sprite(size=32):
    """Create a supply depot sprite (storage building)."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    margin = 2
    
    # Warehouse style building
    draw.rectangle([margin, size//3, size - margin, size - margin], fill=(140, 130, 110, 255))
    
    # Curved/peaked roof
    draw.polygon([
        (margin, size//3),
        (size//2, margin + 2),
        (size - margin, size//3)
    ], fill=(120, 80, 60, 255))
    
    # Large cargo door
    door_w = size - 8
    draw.rectangle([4, size//2, size - 4, size - margin], fill=(100, 90, 70, 255))
    
    # Crates visible through door
    draw.rectangle([6, size//2 + 4, 12, size - 4], fill=(180, 140, 80, 255))
    draw.rectangle([14, size//2 + 6, 20, size - 4], fill=(160, 120, 60, 255))
    draw.rectangle([22, size//2 + 4, 28, size - 4], fill=(170, 130, 70, 255))
    
    return img


def create_tech_lab_sprite(size=40):
    """Create a tech lab building sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    margin = 3
    cx = size // 2
    
    # Modern/futuristic building
    draw.rectangle([margin, size//4, size - margin, size - margin], fill=(80, 70, 120, 255))
    
    # Dome on top
    draw.ellipse([cx - size//4, margin, cx + size//4, size//3], fill=(100, 90, 150, 255))
    
    # Glowing windows
    win_color = (100, 200, 255, 255)
    for i in range(2):
        for j in range(2):
            wx = margin + 6 + i * 16
            wy = size//3 + 4 + j * 10
            draw.ellipse([wx, wy, wx + 8, wy + 6], fill=win_color)
    
    # Central energy core (glowing)
    draw.ellipse([cx - 4, size//5, cx + 4, size//5 + 8], fill=(150, 100, 255, 255))
    
    # Satellite dish
    draw.arc([size - margin - 12, margin, size - margin, margin + 10], 180, 360, fill=(150, 150, 150, 255), width=2)
    draw.line([size - margin - 6, margin + 5, size - margin - 6, margin + 12], fill=(100, 100, 100, 255), width=1)
    
    return img


def create_turret_sprite(size=24):
    """Create a defensive turret sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx, cy = size // 2, size // 2
    
    # Base (octagonal platform)
    base_r = size // 3
    draw.ellipse([cx - base_r, size - 8, cx + base_r, size - 2], fill=(80, 80, 80, 255))
    
    # Support column
    col_w = 4
    draw.rectangle([cx - col_w, cy, cx + col_w, size - 6], fill=(100, 100, 100, 255))
    
    # Turret head (rotating part)
    head_r = size // 4
    draw.ellipse([cx - head_r, cy - head_r + 2, cx + head_r, cy + head_r + 2], fill=(120, 120, 120, 255))
    
    # Gun barrel
    draw.rectangle([cx - 2, 2, cx + 2, cy], fill=(60, 60, 60, 255))
    
    # Muzzle flash area
    draw.ellipse([cx - 3, 0, cx + 3, 5], fill=(255, 200, 100, 200))
    
    return img


def create_resource_node_sprite(size=40, permanent=False):
    """Create a resource node sprite."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    cx, cy = size // 2, size // 2
    
    if permanent:
        # Permanent node - crystalline/rich deposit
        # Central crystal
        color = (60, 200, 100, 255)
        draw.polygon([
            (cx, 4),
            (cx + 10, cy - 4),
            (cx + 8, size - 8),
            (cx - 8, size - 8),
            (cx - 10, cy - 4)
        ], fill=color)
        
        # Side crystals
        draw.polygon([
            (6, cy),
            (14, 8),
            (16, cy + 6)
        ], fill=(80, 220, 120, 255))
        
        draw.polygon([
            (size - 6, cy),
            (size - 14, 10),
            (size - 16, cy + 4)
        ], fill=(50, 180, 90, 255))
        
        # Glow effect
        draw.ellipse([cx - 6, cy - 2, cx + 6, cy + 8], fill=(100, 255, 150, 100))
    else:
        # Temporary node - ore pile
        color = (220, 180, 60, 255)
        
        # Pile of ore chunks
        draw.ellipse([4, size//2, size - 4, size - 4], fill=(180, 140, 40, 255))
        
        # Individual chunks on top
        for i in range(5):
            ox = 8 + (i % 3) * 10 + (i // 3) * 5
            oy = size//3 + (i % 2) * 8
            chunk_size = 8 + (i % 3) * 2
            draw.ellipse([ox, oy, ox + chunk_size, oy + chunk_size], fill=color)
        
        # Metallic glints
        draw.ellipse([cx - 2, cy - 4, cx + 2, cy], fill=(255, 240, 180, 255))
        draw.ellipse([cx + 6, cy + 2, cx + 9, cy + 5], fill=(255, 230, 160, 255))
    
    return img


def create_terrain_sprite(size=80):
    """Create a terrain obstacle sprite (rocks/debris)."""
    img = Image.new("RGBA", (size, 60), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Large rock formation
    draw.polygon([
        (10, 55),
        (5, 35),
        (15, 15),
        (35, 8),
        (55, 12),
        (70, 20),
        (75, 40),
        (70, 55)
    ], fill=(100, 95, 85, 255))
    
    # Highlights
    draw.polygon([
        (20, 45),
        (18, 30),
        (30, 18),
        (45, 20),
        (48, 35)
    ], fill=(120, 115, 105, 255))
    
    # Shadow areas
    draw.polygon([
        (50, 50),
        (55, 25),
        (68, 30),
        (70, 50)
    ], fill=(80, 75, 65, 255))
    
    return img


def apply_faction_tint(img, faction_color, intensity=0.4):
    """Apply a faction color tint to an image."""
    result = img.copy()
    pixels = result.load()
    
    for y in range(result.height):
        for x in range(result.width):
            r, g, b, a = pixels[x, y]
            if a > 0:  # Only tint non-transparent pixels
                # Blend with faction color
                nr = int(r * (1 - intensity) + faction_color[0] * intensity)
                ng = int(g * (1 - intensity) + faction_color[1] * intensity)
                nb = int(b * (1 - intensity) + faction_color[2] * intensity)
                pixels[x, y] = (nr, ng, nb, a)
    
    return result


def main():
    """Generate all sprites."""
    print("Generating sprites...")
    
    # Base sprites (faction-neutral)
    sprites = {
        "infantry": create_infantry_sprite(32),
        "ranger": create_ranger_sprite(32),
        "harvester": create_harvester_sprite(36),
        "depot": create_depot_sprite(64),
        "barracks": create_barracks_sprite(48),
        "supply_depot": create_supply_depot_sprite(32),
        "tech_lab": create_tech_lab_sprite(40),
        "turret": create_turret_sprite(24),
        "resource_temp": create_resource_node_sprite(40, permanent=False),
        "resource_perm": create_resource_node_sprite(40, permanent=True),
        "terrain": create_terrain_sprite(80),
    }
    
    # Save base sprites
    for name, img in sprites.items():
        path = os.path.join(OUTPUT_DIR, f"{name}.png")
        img.save(path)
        print(f"  Created {path}")
    
    # Create faction-tinted versions for units
    factions = {
        "continuity": CONTINUITY_BLUE,
        "collegium": COLLEGIUM_GOLD,
    }
    
    unit_sprites = ["infantry", "ranger", "harvester"]
    building_sprites = ["depot", "barracks", "supply_depot", "tech_lab", "turret"]
    
    for faction_name, faction_color in factions.items():
        faction_dir = os.path.join(OUTPUT_DIR, faction_name)
        os.makedirs(faction_dir, exist_ok=True)
        
        for sprite_name in unit_sprites + building_sprites:
            tinted = apply_faction_tint(sprites[sprite_name], faction_color, 0.35)
            path = os.path.join(faction_dir, f"{sprite_name}.png")
            tinted.save(path)
            print(f"  Created {path}")
    
    print(f"\nAll sprites saved to {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
