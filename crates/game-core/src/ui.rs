use super::*;

impl LurhookGame {
    pub(super) fn tile_style(&self, tile: TileKind, visible: bool) -> (char, RGB) {
        let (glyph, color) = match tile {
            TileKind::Land => ('.', self.palette.land),
            TileKind::ShallowWater => ('~', self.palette.shallow),
            TileKind::DeepWater => ('≈', self.palette.deep),
        };
        let color = if visible { color } else { color * 0.4 };
        (glyph, color)
    }

    pub(super) fn draw_map(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for y in 0..VIEW_HEIGHT {
            for x in 0..VIEW_WIDTH {
                let mx = cam_x + x;
                let my = cam_y + y;
                let pt = common::Point::new(mx, my);
                let idx = self.map.idx(pt);
                let tile = self.map.tiles[idx];
                let visible = self.is_visible(pt);
                let (glyph, color) = self.tile_style(tile, visible);
                ctx.set(x, y, color, RGB::named(BLACK), to_cp437(glyph));
            }
        }
        if let GameMode::Aiming { target } = self.mode {
            if target.x >= cam_x
                && target.x < cam_x + VIEW_WIDTH
                && target.y >= cam_y
                && target.y < cam_y + VIEW_HEIGHT
            {
                ctx.set(
                    target.x - cam_x,
                    target.y - cam_y,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    to_cp437('*'),
                );
            }
        }
        if let Some(path) = &self.cast_path {
            for (i, pt) in path.iter().enumerate() {
                if i >= self.cast_step {
                    break;
                }
                if pt.x >= cam_x && pt.x < cam_x + VIEW_WIDTH && pt.y >= cam_y && pt.y < cam_y + VIEW_HEIGHT {
                    let glyph = if i == path.len() - 1 { 'o' } else { '*' };
                    ctx.set(
                        pt.x - cam_x,
                        pt.y - cam_y,
                        RGB::named(WHITE),
                        RGB::named(BLACK),
                        to_cp437(glyph),
                    );
                }
            }
        }
    }

    pub(super) fn draw_fish(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for fish in &self.fishes {
            if fish.position.x >= cam_x
                && fish.position.x < cam_x + VIEW_WIDTH
                && fish.position.y >= cam_y
                && fish.position.y < cam_y + VIEW_HEIGHT
                && self.is_visible(fish.position)
            {
                ctx.set(
                    fish.position.x - cam_x,
                    fish.position.y - cam_y,
                    self.palette.fish,
                    RGB::named(BLACK),
                    to_cp437('f'),
                );
            }
        }
    }

    pub(super) fn draw_hazards(&self, ctx: &mut BTerm) {
        let (cam_x, cam_y) = self.camera();
        for h in &self.hazards {
            if h.pos.x >= cam_x
                && h.pos.x < cam_x + VIEW_WIDTH
                && h.pos.y >= cam_y
                && h.pos.y < cam_y + VIEW_HEIGHT
                && self.is_visible(h.pos)
            {
                ctx.set(
                    h.pos.x - cam_x,
                    h.pos.y - cam_y,
                    self.palette.hazard,
                    RGB::named(BLACK),
                    to_cp437('!'),
                );
            }
        }
    }
}
