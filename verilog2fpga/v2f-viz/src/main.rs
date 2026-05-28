use std::collections::HashMap;
use std::path::PathBuf;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2, vec2};

// ---------------------------------------------------------------------------
// Netlist types (parsed from JSON)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Cell {
    cell_type: String,
    ports: HashMap<String, Vec<u64>>,
}

#[derive(Clone)]
struct Netlist {
    cells: HashMap<String, Cell>,
    netnames: HashMap<String, Vec<u64>>,
    ports: HashMap<String, Port>,
}

#[derive(Clone)]
struct Port {
    direction: String,
    bits: Vec<u64>,
    hide_name: Option<u8>,
}

// ---------------------------------------------------------------------------
// Layout types (parsed from ASC)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct TilePlacement {
    x: u32,
    y: u32,
    cell_name: String,
}

#[derive(Clone, Debug)]
struct WiringSeg {
    from_row: u32,
    from_col: u32,
    to_row: u32,
    to_col: u32,
    track: u32,
}

#[derive(Clone, Debug)]
struct Layout {
    device: String,
    tiles: Vec<TilePlacement>,
    wiring: Vec<WiringSeg>,
    cols: u32,
    rows: u32,
}

// ---------------------------------------------------------------------------
// View mode
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
enum ViewMode {
    Circuit,
    Layout,
    Routing,
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

struct App {
    json_path: Option<PathBuf>,
    asc_path: Option<PathBuf>,
    netlist: Option<Netlist>,
    layout: Option<Layout>,
    view: ViewMode,
    pan: Vec2,
    zoom: f32,
    selected_cell: Option<String>,
    loading_msg: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            json_path: None,
            asc_path: None,
            netlist: None,
            layout: None,
            view: ViewMode::Circuit,
            pan: Vec2::ZERO,
            zoom: 1.0,
            selected_cell: None,
            loading_msg: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag-and-drop
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(p) = &file.path {
                    App::handle_dropped_file(self, p);
                }
            }
        });

        // --- top bar ---
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("檔案", |ui| {
                    if ui.button("開啟 JSON (netlist)...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                        {
                            App::load_json(self, &path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("開啟 ASC (佈局)...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("ASC", &["asc"])
                            .pick_file()
                        {
                            App::load_asc(self, &path);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("結束").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.separator();
                ui.selectable_value(&mut self.view, ViewMode::Circuit, "電路");
                ui.selectable_value(&mut self.view, ViewMode::Layout, "佈局");
                ui.selectable_value(&mut self.view, ViewMode::Routing, "繞線");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("重置視圖").clicked() {
                        self.pan = Vec2::ZERO;
                        self.zoom = 1.0;
                    }
                    ui.label(format!("縮放: {:.0}%", self.zoom * 100.0));
                });
            });
        });

        // --- status bar ---
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.loading_msg.is_empty() {
                    ui.colored_label(Color32::RED, &self.loading_msg);
                }
                if let Some(ref n) = self.netlist {
                    ui.label(format!("cells: {}", n.cells.len()));
                }
                if let Some(ref l) = self.layout {
                    ui.label(format!("  |  tiles: {}/{}", l.tiles.len(), l.cols * l.rows));
                }
                if let Some(ref sel) = self.selected_cell {
                    ui.separator();
                    ui.colored_label(Color32::YELLOW, format!("選中: {sel}"));
                }
            });
        });

        // --- main canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );

            if response.dragged_by(egui::PointerButton::Primary) {
                self.pan += response.drag_delta();
            }

            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
                    self.zoom = (self.zoom * factor).clamp(0.05, 10.0);
                }
            }

            let to_screen = |p: Pos2| -> Pos2 {
                let center = ui.clip_rect().center().to_vec2() + self.pan;
                (p.to_vec2() * self.zoom + center).to_pos2()
            };

            match self.view {
                ViewMode::Circuit => {
                    if let Some(ref net) = self.netlist {
                        draw_circuit(&painter, net, to_screen, self.zoom);
                    } else {
                        let c = ui.clip_rect().center();
                        painter.text(c, egui::Align2::CENTER_CENTER, "請開啟 JSON 檔案\n或拖放至此視窗",
                            egui::FontId::proportional(18.0), Color32::GRAY);
                    }
                }
                ViewMode::Layout => {
                    if let Some(ref lay) = self.layout {
                        draw_placement(&painter, lay, to_screen, self.zoom, &mut self.selected_cell, &response);
                    } else {
                        let c = ui.clip_rect().center();
                        painter.text(c, egui::Align2::CENTER_CENTER, "請開啟 ASC 檔案\n或拖放至此視窗",
                            egui::FontId::proportional(18.0), Color32::GRAY);
                    }
                }
                ViewMode::Routing => {
                    if let (Some(ref net), Some(ref lay)) = (&self.netlist, &self.layout) {
                        draw_routing(&painter, net, lay, to_screen, self.zoom);
                    } else {
                        let c = ui.clip_rect().center();
                        painter.text(c, egui::Align2::CENTER_CENTER, "請開啟 JSON 與 ASC 檔案\n或拖放至此視窗",
                            egui::FontId::proportional(18.0), Color32::GRAY);
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}

// ---------------------------------------------------------------------------
// File loading
// ---------------------------------------------------------------------------

impl App {
    fn load_json(&mut self, path: &std::path::Path) {
        match std::fs::read_to_string(path) {
            Ok(text) => match parse_netlist(&text) {
                Ok(n) => {
                    self.netlist = Some(n);
                    self.json_path = Some(path.to_path_buf());
                    self.loading_msg.clear();
                }
                Err(e) => self.loading_msg = format!("JSON 錯誤: {e}"),
            },
            Err(e) => self.loading_msg = format!("讀取失敗: {e}"),
        }
    }

    fn load_asc(&mut self, path: &std::path::Path) {
        match std::fs::read_to_string(path) {
            Ok(text) => match parse_layout(&text) {
                Ok(l) => {
                    self.layout = Some(l);
                    self.asc_path = Some(path.to_path_buf());
                    self.loading_msg.clear();
                }
                Err(e) => self.loading_msg = format!("ASC 錯誤: {e}"),
            },
            Err(e) => self.loading_msg = format!("讀取失敗: {e}"),
        }
    }

    fn handle_dropped_file(&mut self, path: &std::path::Path) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "json" => self.load_json(path),
            "asc" => self.load_asc(path),
            _ => self.loading_msg = format!("不支援的檔案: {}", path.display()),
        }
    }
}

// ---------------------------------------------------------------------------
// Netlist parser
// ---------------------------------------------------------------------------

fn parse_netlist(text: &str) -> Result<Netlist, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let modules = v["modules"].as_object().ok_or("缺少 modules")?;
    let (_name, module) = modules.iter().next().ok_or("沒有 module")?;

    let mut cells = HashMap::new();
    if let Some(cells_obj) = module["cells"].as_object() {
        for (name, cell) in cells_obj {
            let cell_type = cell["type"].as_str().unwrap_or("?").to_string();
            let mut ports = HashMap::new();
            if let Some(conns) = cell["connections"].as_object() {
                for (pname, bits) in conns {
                    let bits_vec: Vec<u64> = bits.as_array().ok_or("bits not array")?
                        .iter().map(|b| b.as_u64().unwrap_or(0)).collect();
                    ports.insert(pname.clone(), bits_vec);
                }
            }
            cells.insert(name.clone(), Cell { cell_type, ports });
        }
    }

    let mut netnames = HashMap::new();
    if let Some(nets_obj) = module["netnames"].as_object() {
        for (name, net) in nets_obj {
            let bits: Vec<u64> = net["bits"].as_array().ok_or("net bits not array")?
                .iter().map(|b| b.as_u64().unwrap_or(0)).collect();
            let hide_name = net.get("hide_name").and_then(|v| v.as_u64()).map(|v| v as u8);
            netnames.insert(name.clone(), bits);
            // We won't use hide_name for now, but it's parsed for completeness
            let _ = hide_name;
        }
    }

    let mut ports = HashMap::new();
    if let Some(ports_obj) = module["ports"].as_object() {
        for (name, port) in ports_obj {
            let direction = port["direction"].as_str().unwrap_or("?").to_string();
            let bits: Vec<u64> = port["bits"].as_array().ok_or("port bits not array")?
                .iter().map(|b| b.as_u64().unwrap_or(0)).collect();
            ports.insert(name.clone(), Port { direction, bits, hide_name: None });
        }
    }

    Ok(Netlist { cells, netnames, ports })
}

// ---------------------------------------------------------------------------
// Layout parser (ASC)
// ---------------------------------------------------------------------------

fn parse_layout(text: &str) -> Result<Layout, String> {
    let mut device = String::new();
    let mut tiles = Vec::new();
    let mut wiring = Vec::new();
    let mut pending: Option<(u32, u32)> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with(".device ") {
            device = line.trim_start_matches(".device ").trim().to_string();
        } else if line.starts_with(".logic_tile ") {
            let rest = line.trim_start_matches(".logic_tile ").trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let x: u32 = parts[0].parse().map_err(|_| format!("bad tile x: {}", parts[0]))?;
                let y: u32 = parts[1].parse().map_err(|_| format!("bad tile y: {}", parts[1]))?;
                if parts.len() >= 6 {
                    let cell_name = parts[5].trim_matches('"').to_string();
                    tiles.push(TilePlacement { x, y, cell_name });
                } else {
                    pending = Some((x, y));
                }
            }
        } else if line.starts_with(".sym") && pending.is_some() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                if let Some((x, y)) = pending.take() {
                    let cell_name = parts[5].trim_matches('"').to_string();
                    tiles.push(TilePlacement { x, y, cell_name });
                }
            }
        } else if line.starts_with(".wiring ") {
            let rest = line.trim_start_matches(".wiring ").trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 4 {
                let from_col: u32 = parts[0].parse().map_err(|_| format!("bad wiring col: {}", parts[0]))?;
                let from_row: u32 = parts[1].parse().map_err(|_| format!("bad wiring row: {}", parts[1]))?;
                let to_col: u32 = parts[2].parse().map_err(|_| format!("bad wiring to_col: {}", parts[2]))?;
                let track: u32 = parts[3].parse().map_err(|_| format!("bad wiring track: {}", parts[3]))?;
                let to_row = if from_col == to_col { from_row + 1 } else { from_row };
                wiring.push(WiringSeg { from_row, from_col, to_row, to_col, track });
            }
        } else {
            pending = None;
        }
    }

    // Grid size from Ice40Device (rows = num_rows - 2 for IO tile margins)
    let (cols, rows) = match device.as_str() {
        d if d.contains("HX1K") => (16, 28),
        d if d.contains("LP1K") => (16, 28),
        d if d.contains("HX4K") => (20, 38),
        d if d.contains("HX8K") => (28, 68),
        d if d.contains("UP5K") => (22, 38),
        _ => {
            let mc = tiles.iter().map(|t| t.x).max().unwrap_or(31);
            let mr = tiles.iter().map(|t| t.y).max().unwrap_or(19);
            (mc + 1, mr + 1)
        }
    };

    Ok(Layout { device, tiles, wiring, cols, rows })
}

// ---------------------------------------------------------------------------
// Circuit drawing
// ---------------------------------------------------------------------------

fn draw_circuit(
    painter: &egui::Painter,
    net: &Netlist,
    to_screen: impl Fn(Pos2) -> Pos2,
    zoom: f32,
) {
    let layer_order = ["$_INPUT_", "$_OUTPUT_", "$_ONE_", "$_ADD_", "$_DFF_P_", "$_AND_"];
    let mut layers: HashMap<&str, Vec<(&String, &Cell)>> = HashMap::new();
    for &l in &layer_order {
        layers.insert(l, Vec::new());
    }
    for (name, cell) in &net.cells {
        layers.entry(cell.cell_type.as_str()).or_insert_with(Vec::new).push((name, cell));
    }

    let cell_w = 120.0 * zoom;
    let cell_h = 50.0 * zoom;
    let gap_x = 80.0 * zoom;
    let gap_y = 30.0 * zoom;

    let mut positions: HashMap<String, Pos2> = HashMap::new();

    let mut col = 0.0_f32;
    for &layer_name in &layer_order {
        if let Some(cells) = layers.get(layer_name) {
            if cells.is_empty() { continue; }
            let n = cells.len() as f32;
            let total_h = n * cell_h + (n - 1.0) * gap_y;
            let mut row_offset = -total_h / 2.0;

            for (name, _cell) in cells {
                let px = col;
                let py = row_offset + cell_h / 2.0;
                positions.insert(name.to_string(), Pos2::new(px, py));
                row_offset += cell_h + gap_y;
            }
            col += cell_w + gap_x;
        }
    }

    // Build net → cell mapping
    let mut net_to_cells: HashMap<u64, Vec<(String, String)>> = HashMap::new();
    for (name, cell) in &net.cells {
        for (port, bits) in &cell.ports {
            for &b in bits {
                net_to_cells.entry(b).or_default().push((name.clone(), port.clone()));
            }
        }
    }

    // Draw edges
    let stroke = Stroke::new(1.5 * zoom, Color32::from_rgba_premultiplied(137, 180, 250, 120));
    for (_net_id, conns) in &net_to_cells {
        if conns.len() < 2 { continue; }
        for i in 0..conns.len() {
            for j in i + 1..conns.len() {
                let (name_a, _port_a) = &conns[i];
                let (name_b, _port_b) = &conns[j];
                if let (Some(&pa), Some(&pb)) = (positions.get(name_a), positions.get(name_b)) {
                    let from = to_screen(pa);
                    let to = to_screen(pb);
                    let mid = from.lerp(to, 0.5);
                    let cp = mid + vec2(0.0, 30.0 * zoom);
                    let points = subdivide_bezier(from, cp, to, 20);
                    painter.add(egui::Shape::line(points, stroke));
                }
            }
        }
    }

    // Draw cells
    for (name, cell) in &net.cells {
        if let Some(&pos) = positions.get(name) {
            let screen_pos = to_screen(pos);
            let rect = Rect::from_center_size(screen_pos, vec2(cell_w, cell_h));
            let color = cell_color(&cell.cell_type);
            painter.rect(rect, 4.0_f32, color, Stroke::new(1.0_f32, Color32::WHITE));
            painter.text(
                rect.left_center() + vec2(6.0, 0.0),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional((9.0 * zoom).max(4.0)),
                Color32::WHITE,
            );
            painter.text(
                rect.right_center() - vec2(6.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                &cell.cell_type,
                egui::FontId::proportional((8.0 * zoom).max(4.0)),
                Color32::LIGHT_GRAY,
            );
        }
    }
}

fn subdivide_bezier(p0: Pos2, p1: Pos2, p2: Pos2, n: usize) -> Vec<Pos2> {
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let x = mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x;
        let y = mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y;
        pts.push(Pos2::new(x, y));
    }
    pts
}

fn cell_color(cell_type: &str) -> Color32 {
    match cell_type {
        "$_INPUT_" => Color32::from_rgb(70, 70, 120),
        "$_OUTPUT_" => Color32::from_rgb(120, 60, 60),
        "$_ONE_" => Color32::from_rgb(80, 90, 60),
        "$_ADD_" => Color32::from_rgb(60, 90, 120),
        "$_DFF_P_" => Color32::from_rgb(60, 110, 70),
        "$_AND_" => Color32::from_rgb(100, 70, 100),
        _ => Color32::DARK_GRAY,
    }
}

// ---------------------------------------------------------------------------
// Placement drawing
// ---------------------------------------------------------------------------

fn draw_placement(
    painter: &egui::Painter,
    layout: &Layout,
    to_screen: impl Fn(Pos2) -> Pos2,
    _zoom: f32,
    selected: &mut Option<String>,
    response: &egui::Response,
) {
    let tile_w = 14.0_f32;
    let tile_h = 14.0_f32;
    let gap = 2.0_f32;
    let step_x = tile_w + gap;
    let step_y = tile_h + gap;

    let mut tile_map: HashMap<(u32, u32), &TilePlacement> = HashMap::new();
    for t in &layout.tiles {
        tile_map.insert((t.x, t.y), t);
    }

    let origin = Pos2::new(
        -(layout.cols as f32 * step_x) / 2.0,
        -(layout.rows as f32 * step_y) / 2.0,
    );

    for row in 0..layout.rows {
        for col in 0..layout.cols {
            let pos = Pos2::new(
                origin.x + col as f32 * step_x,
                origin.y + row as f32 * step_y,
            );
            let screen = to_screen(pos);
            let is_used = tile_map.contains_key(&(col as u32, row as u32));
            let tile = tile_map.get(&(col as u32, row as u32));

            let color = if is_used {
                Color32::from_rgb(80, 150, 80)
            } else {
                Color32::from_rgba_premultiplied(40, 40, 50, 200)
            };

            let rect = Rect::from_min_size(screen, vec2(tile_w, tile_h));
            painter.rect(rect, 2.0_f32, color, Stroke::new(0.5_f32, Color32::from_gray(60)));

            if let Some(t) = tile {
                if selected.as_deref() == Some(&t.cell_name) {
                    painter.rect_stroke(rect.expand(1.0), 2.0_f32, Stroke::new(2.0_f32, Color32::YELLOW));
                }
                let mouse_pos = response.interact_pointer_pos();
                if let Some(mp) = mouse_pos {
                    if rect.contains(mp) {
                        *selected = Some(t.cell_name.clone());
                        painter.text(
                            rect.left_top() - vec2(0.0, 16.0),
                            egui::Align2::LEFT_BOTTOM,
                            &t.cell_name,
                            egui::FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                        painter.text(
                            rect.left_top() - vec2(0.0, 28.0),
                            egui::Align2::LEFT_BOTTOM,
                            &format!("({}, {})", t.x, t.y),
                            egui::FontId::proportional(9.0),
                            Color32::LIGHT_GRAY,
                        );
                    }
                }
            }
        }
    }

    // IO port labels
    for t in &layout.tiles {
        if t.cell_name.starts_with("port_") {
            let pos = Pos2::new(
                origin.x + t.x as f32 * step_x,
                origin.y + t.y as f32 * step_y,
            );
            let screen = to_screen(pos);
            painter.text(
                screen - vec2(0.0, 8.0),
                egui::Align2::CENTER_BOTTOM,
                &t.cell_name,
                egui::FontId::proportional(10.0),
                Color32::YELLOW,
            );
        }
    }

    let legend_x = to_screen(Pos2::new(origin.x, origin.y + layout.rows as f32 * step_y + 30.0));
    painter.text(legend_x, egui::Align2::LEFT_CENTER,
        &format!("{} {}  tiles: {}/{}",
            layout.device, layout.cols, layout.tiles.len(), layout.cols * layout.rows),
        egui::FontId::proportional(11.0), Color32::GRAY);
}

// ---------------------------------------------------------------------------
// Routing drawing
// ---------------------------------------------------------------------------

fn draw_routing(
    painter: &egui::Painter,
    net: &Netlist,
    layout: &Layout,
    to_screen: impl Fn(Pos2) -> Pos2,
    zoom: f32,
) {
    let tile_w = 14.0_f32;
    let tile_h = 14.0_f32;
    let gap = 2.0_f32;
    let step_x = tile_w + gap;
    let step_y = tile_h + gap;

    let origin = Pos2::new(
        -(layout.cols as f32 * step_x) / 2.0,
        -(layout.rows as f32 * step_y) / 2.0,
    );

    let tile_center = |tx: u32, ty: u32| -> Pos2 {
        Pos2::new(origin.x + tx as f32 * step_x + step_x / 2.0,
                  origin.y + ty as f32 * step_y + step_y / 2.0)
    };

    // Step 1: background grid (drawn first, lines on top)
    for row in 0..layout.rows {
        for col in 0..layout.cols {
            let pos = Pos2::new(
                origin.x + col as f32 * step_x,
                origin.y + row as f32 * step_y,
            );
            let screen = to_screen(pos);
            let rect = Rect::from_min_size(screen, vec2(tile_w, tile_h));
            let color = Color32::from_rgba_premultiplied(30, 30, 40, 100);
            painter.rect(rect, 1.0_f32, color, Stroke::new(0.3_f32, Color32::from_gray(40)));
        }
    }

    // Step 2: draw physical wiring segments from ASC
    let track_colors = [
        Color32::from_rgb(255, 100, 100),
        Color32::from_rgb(100, 200, 255),
        Color32::from_rgb(100, 255, 100),
        Color32::from_rgb(255, 255, 100),
        Color32::from_rgb(255, 150, 50),
        Color32::from_rgb(200, 100, 255),
        Color32::from_rgb(255, 100, 200),
        Color32::from_rgb(100, 255, 200),
    ];

    for w in &layout.wiring {
        let from = to_screen(tile_center(w.from_col, w.from_row));
        let to = to_screen(tile_center(w.to_col, w.to_row));
        let color = track_colors[(w.track as usize) % track_colors.len()];
        painter.line_segment([from, to], Stroke::new(2.5 * zoom, color));
    }

    // Step 3: draw logical connectivity overlay (all-pairs)
    let mut cell_tile: HashMap<String, (u32, u32)> = HashMap::new();
    for t in &layout.tiles {
        cell_tile.insert(t.cell_name.clone(), (t.x, t.y));
    }

    let mut net_to_cells: HashMap<u64, Vec<String>> = HashMap::new();
    for (name, cell) in &net.cells {
        for (_port, bits) in &cell.ports {
            for &b in bits {
                net_to_cells.entry(b).or_default().push(name.clone());
            }
        }
    }

    let net_colors = [
        Color32::from_rgba_premultiplied(137, 180, 250, 60),
        Color32::from_rgba_premultiplied(166, 227, 161, 60),
        Color32::from_rgba_premultiplied(249, 226, 175, 60),
        Color32::from_rgba_premultiplied(243, 139, 168, 60),
        Color32::from_rgba_premultiplied(203, 166, 247, 60),
    ];
    let mut color_idx = 0usize;

    for (_net_id, cells) in &net_to_cells {
        if cells.len() < 2 { continue; }
        let mut positions: Vec<Pos2> = Vec::new();
        for cname in cells {
            if let Some(&(tx, ty)) = cell_tile.get(cname) {
                positions.push(tile_center(tx, ty));
            }
        }
        if positions.len() < 2 { continue; }
        let color = net_colors[color_idx % net_colors.len()];
        color_idx += 1;

        for i in 0..positions.len() {
            for j in i + 1..positions.len() {
                let from = to_screen(positions[i]);
                let to = to_screen(positions[j]);
                painter.line_segment([from, to], Stroke::new(0.8 * zoom, color));
            }
        }
    }

    // Step 4: used-tile highlights
    let mut tile_map: HashMap<(u32, u32), &TilePlacement> = HashMap::new();
    for t in &layout.tiles {
        tile_map.insert((t.x, t.y), t);
    }
    for t in &layout.tiles {
        let center = to_screen(tile_center(t.x, t.y));
        let rect = Rect::from_center_size(center, vec2(tile_w * 0.8, tile_h * 0.8));
        painter.rect(rect, 2.0_f32, Color32::from_rgba_premultiplied(60, 120, 60, 80),
                     Stroke::new(0.5_f32, Color32::from_rgba_premultiplied(100, 180, 100, 120)));
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("v2f-viz — FPGA 電路視覺化"),
        ..Default::default()
    };

    eframe::run_native("v2f-viz", options, Box::new(|cc| {
        let mut app = App::default();

        // Load files from CLI args
        for arg in args.iter().skip(1) {
            let path = PathBuf::from(arg);
            if path.exists() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "json" {
                    app.load_json(&path);
                } else if ext == "asc" {
                    app.load_asc(&path);
                }
            }
        }

        // Load Chinese font
        if let Ok(font_data) = std::fs::read("font/font.ttf") {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
            fonts.families.entry(egui::FontFamily::Monospace).or_default().push("chinese".to_owned());
            cc.egui_ctx.set_fonts(fonts);
        }

        // Customize the visual style for dark theme
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_rounding = 4.0.into();
        cc.egui_ctx.set_style(style);

        Ok(Box::new(app))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blinky_json() {
        let json = r#"{
            "creator": "v2f-synth v0.3",
            "modules": {
                "blinky": {
                    "cells": {
                        "$0": { "type": "$_ONE_", "connections": { "Y": [29] }, "parameters": {}, "port_directions": { "Y": "output" } },
                        "$27": { "type": "$_DFF_P_", "connections": { "D": [30], "Q": [56] }, "parameters": {}, "port_directions": { "C": "input", "D": "input", "Q": "output" } },
                        "$80": { "type": "$_INPUT_", "connections": { "Y": [1] }, "parameters": {}, "port_directions": { "Y": "output" } },
                        "$81": { "type": "$_OUTPUT_", "connections": { "A": [2], "Y": [2] }, "parameters": {}, "port_directions": { "A": "input", "Y": "output" } }
                    },
                    "netnames": {
                        "clk": { "bits": [1] },
                        "led": { "bits": [2] }
                    },
                    "ports": {
                        "clk": { "bits": [1], "direction": "input" },
                        "led": { "bits": [2], "direction": "output" }
                    }
                }
            }
        }"#;
        let net = parse_netlist(json).unwrap();
        assert_eq!(net.cells.len(), 4);
        assert!(net.cells.contains_key("$0"));
        assert!(net.cells.contains_key("$27"));
        assert!(net.cells.contains_key("$80"));
        assert!(net.cells.contains_key("$81"));
        assert_eq!(net.ports.len(), 2);
        assert!(net.ports.contains_key("clk"));
        assert!(net.ports.contains_key("led"));
    }

    #[test]
    fn test_parse_blinky_asc() {
        let asc = ".device HX8K-CT256\n
            .logic_tile 3 20 0 0 0 \"$59\"\n
            .logic_tile 33 1 0 0 0 \"$51\"\n
            .logic_tile 35 9 0 0 0 \"port_clk\"\n
            .logic_tile 66 3 0 0 0 \"port_led\"\n";
        let lay = parse_layout(asc).unwrap();
        assert_eq!(lay.device, "HX8K-CT256");
        assert_eq!(lay.tiles.len(), 4);
        assert_eq!(lay.tiles[2].cell_name, "port_clk");
        assert_eq!(lay.tiles[3].cell_name, "port_led");
        assert_eq!(lay.cols, 28);
        assert_eq!(lay.rows, 68);
    }
}
