use eframe::egui;
use tokio::sync::broadcast;

use crate::node::node::NodeEvent;
use crate::types::block::Block;

pub struct BlockchainViewer {
    blocks: Vec<Block>,
    rx: broadcast::Receiver<NodeEvent>,
    offset: egui::Vec2,
    is_dragging: bool,
    selected_block: Option<usize>,
    zoom: f32,
}

impl BlockchainViewer {
    pub fn new(rx: broadcast::Receiver<NodeEvent>) -> Self {
        Self {
            blocks: Vec::new(),
            rx,
            offset: egui::Vec2::new(50.0, 0.0),
            is_dragging: false,
            selected_block: None,
            zoom: 1.0,
        }
    }

    fn get_block_position(&self, height: usize, available_rect: egui::Rect) -> egui::Pos2 {
        let block_width = 180.0 * self.zoom;
        let block_spacing = 100.0 * self.zoom;
        let total_block_width = block_width + block_spacing;
        let row_height = 300.0 * self.zoom;
        
        let row = height / 10;
        let col = height % 10;
        
        // Reverse direction for odd rows (snake pattern)
        let x_col = if row % 2 == 1 {
            9 - col // Right to left
        } else {
            col // Left to right
        };
        
        let x = available_rect.min.x + self.offset.x + (x_col as f32 * total_block_width);
        let y = available_rect.min.y + 100.0 + self.offset.y + (row as f32 * row_height);
        
        egui::Pos2::new(x, y)
    }

    fn draw_block(&self, ui: &mut egui::Ui, block: &Block, height: usize, pos: egui::Pos2) -> egui::Response {
        let block_size = egui::Vec2::new(180.0 * self.zoom, 200.0 * self.zoom);
        let rect = egui::Rect::from_min_size(pos, block_size);
        
        let response = ui.allocate_rect(rect, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Draw block shape
            let is_selected = self.selected_block == Some(height);
            let color = if is_selected {
                egui::Color32::from_rgb(96, 165, 250)
            } else if response.hovered() {
                egui::Color32::from_rgb(66, 153, 225)
            } else {
                egui::Color32::from_rgb(45, 55, 72)
            };
            
            let stroke_color = if is_selected {
                egui::Color32::from_rgb(147, 197, 253)
            } else {
                egui::Color32::from_rgb(66, 153, 225)
            };
            
            let stroke_width = if is_selected { 4.0 * self.zoom } else { 3.0 * self.zoom };
            
            // Main block body
            painter.rect(
                rect.shrink(5.0 * self.zoom),
                8.0 * self.zoom,
                color,
                egui::Stroke::new(stroke_width, stroke_color),
                egui::epaint::StrokeKind::Outside,
            );
            
            // Draw diagonal corners to make it look more like a 3D block
            let corner_size = 15.0 * self.zoom;
            let top_left = rect.left_top() + egui::Vec2::new(5.0 * self.zoom, 5.0 * self.zoom);
            let top_right = rect.right_top() + egui::Vec2::new(-5.0 * self.zoom, 5.0 * self.zoom);
            
            // Top bevel
            painter.line_segment(
                [top_left, top_left + egui::Vec2::new(corner_size, -corner_size)],
                egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_rgb(96, 165, 250))
            );
            painter.line_segment(
                [top_right, top_right + egui::Vec2::new(corner_size, -corner_size)],
                egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_rgb(96, 165, 250))
            );
            
            // Block content
            let text_rect = rect.shrink(15.0 * self.zoom);
            
            // Height number (large)
            painter.text(
                egui::Pos2::new(rect.center().x, text_rect.min.y + 15.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                format!("#{}", height),
                egui::FontId::proportional(24.0 * self.zoom),
                egui::Color32::from_rgb(147, 197, 253),
            );
            
            // Hash (truncated)
            let hash_str = format!("{:?}", block.hash);
            let hash_display = if hash_str.len() > 16 {
                format!("{}...", &hash_str[..16])
            } else {
                hash_str
            };
            
            painter.text(
                egui::Pos2::new(rect.center().x, text_rect.min.y + 45.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                hash_display,
                egui::FontId::monospace(10.0 * self.zoom),
                egui::Color32::from_rgb(203, 213, 225),
            );
            
            // Nonce
            painter.text(
                egui::Pos2::new(rect.center().x, text_rect.min.y + 75.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                format!("Nonce: {}", block.nonce),
                egui::FontId::proportional(11.0 * self.zoom),
                egui::Color32::from_rgb(251, 191, 36),
            );
            
            // TX count
            painter.text(
                egui::Pos2::new(rect.center().x, text_rect.min.y + 105.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                format!("TX: {}", block.transactions.len()),
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::from_rgb(74, 222, 128),
            );
            
            // Timestamp if available
            if let Some(timestamp) = block.timestamp {
                painter.text(
                    egui::Pos2::new(rect.center().x, text_rect.min.y + 135.0 * self.zoom),
                    egui::Align2::CENTER_TOP,
                    timestamp.format("%H:%M:%S").to_string(),
                    egui::FontId::proportional(10.0 * self.zoom),
                    egui::Color32::from_rgb(156, 163, 175),
                );
            }
        }
        
        response
    }

    fn draw_horizontal_arrow(&self, ui: &mut egui::Ui, from_pos: egui::Pos2, to_pos: egui::Pos2) {
        let painter = ui.painter();
        
        // Draw arrow line
        painter.line_segment(
            [from_pos, to_pos],
            egui::Stroke::new(3.0 * self.zoom, egui::Color32::from_rgb(100, 116, 139)),
        );
        
        // Draw arrowhead
        let arrow_size = 12.0 * self.zoom;
        let direction = (to_pos - from_pos).normalized();
        let perpendicular = egui::Vec2::new(-direction.y, direction.x);
        
        let arrow_tip = to_pos;
        let arrow_left = arrow_tip - direction * arrow_size + perpendicular * (arrow_size / 2.0);
        let arrow_right = arrow_tip - direction * arrow_size - perpendicular * (arrow_size / 2.0);
        
        painter.add(egui::Shape::convex_polygon(
            vec![arrow_tip, arrow_left, arrow_right],
            egui::Color32::from_rgb(100, 116, 139),
            egui::Stroke::NONE,
        ));
    }

    fn draw_wrap_arrow(&self, ui: &mut egui::Ui, from_pos: egui::Pos2, to_pos: egui::Pos2) {
        let painter = ui.painter();
        let stroke = egui::Stroke::new(3.0 * self.zoom, egui::Color32::from_rgb(100, 116, 139));
        
        // Calculate the bend point (down then left/right)
        let mid_y = from_pos.y + 80.0 * self.zoom;
        let mid_point1 = egui::Pos2::new(from_pos.x, mid_y);
        let mid_point2 = egui::Pos2::new(to_pos.x, mid_y);
        
        // Draw vertical line down
        painter.line_segment([from_pos, mid_point1], stroke);
        
        // Draw horizontal line
        painter.line_segment([mid_point1, mid_point2], stroke);
        
        // Draw vertical line up to next block
        painter.line_segment([mid_point2, to_pos], stroke);
        
        // Draw arrowhead pointing up
        let arrow_size = 12.0 * self.zoom;
        let arrow_tip = to_pos;
        let arrow_left = arrow_tip + egui::Vec2::new(-arrow_size / 2.0, arrow_size);
        let arrow_right = arrow_tip + egui::Vec2::new(arrow_size / 2.0, arrow_size);
        
        painter.add(egui::Shape::convex_polygon(
            vec![arrow_tip, arrow_left, arrow_right],
            egui::Color32::from_rgb(100, 116, 139),
            egui::Stroke::NONE,
        ));
    }

    fn draw_block_details(&self, ui: &mut egui::Ui, block: &Block, height: usize) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            
            // Title
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(format!("Block #{}", height))
                    .size(32.0)
                    .color(egui::Color32::from_rgb(147, 197, 253)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("✕ Close").size(16.0)).clicked() {
                        // Will be handled by parent
                    }
                });
            });
            
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // Header Section
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(66, 153, 225)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("📋 BLOCK HEADER")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(147, 197, 253))
                        .strong());
                    
                    ui.add_space(12.0);

                    // Hash
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Hash:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(format!("{:?}", block.hash))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    // Previous Hash
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Previous Hash:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(format!("{:?}", block.previous_hash))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    // Merkle Root
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Merkle Root:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(format!("{:?}", block.merkle_root))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    // Nonce
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Nonce:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                        ui.label(egui::RichText::new(format!("{}", block.nonce))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(251, 191, 36)));
                    });

                    // Timestamp
                    if let Some(timestamp) = block.timestamp {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Timestamp:")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(148, 163, 184))
                                .strong());
                            ui.label(egui::RichText::new(timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                                .size(14.0)
                                .color(egui::Color32::from_rgb(226, 232, 240)));
                        });
                    }
                });

            ui.add_space(20.0);

            // Transactions Section
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(66, 153, 225)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("💳 TRANSACTIONS ({})", block.transactions.len()))
                        .size(18.0)
                        .color(egui::Color32::from_rgb(147, 197, 253))
                        .strong());
                    
                    ui.add_space(12.0);

                    if block.transactions.is_empty() {
                        ui.label(egui::RichText::new("No transactions in this block")
                            .size(13.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .italics());
                    } else {
                        for (i, tx) in block.transactions.iter().enumerate() {
                            egui::Frame::new()
                                .fill(egui::Color32::from_rgb(15, 23, 42))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 65, 85)))
                                .corner_radius(6.0)
                                .inner_margin(12.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("Transaction #{}", i + 1))
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(74, 222, 128))
                                            .strong());
                                    });
                                    ui.add_space(6.0);
                                    ui.monospace(egui::RichText::new(format!("{:?}", tx))
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(203, 213, 225)));
                                });
                            ui.add_space(8.0);
                        }
                    }
                });

            ui.add_space(20.0);
        });
    }
}

impl eframe::App for BlockchainViewer {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Drain ALL events without blocking
        loop {
            match self.rx.try_recv() {
                Ok(NodeEvent::NewBlock(block)) => {
                    self.blocks.push(block);
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                    eprintln!("⚠️ Viewer lagged behind! Skipped {} blocks", skipped);
                    // Continue draining to catch up
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    eprintln!("❌ Event channel closed");
                    break;
                }
            }
        }

        // Show detail panel if a block is selected
        if let Some(selected_height) = self.selected_block {
            egui::SidePanel::right("block_details")
                .min_width(500.0)
                .max_width(700.0)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(block) = self.blocks.get(selected_height) {
                        self.draw_block_details(ui, block, selected_height);
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar
            ui.horizontal(|ui| {
                ui.heading("🔗 Blockchain Explorer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.selected_block.is_some() {
                        if ui.button(egui::RichText::new("✕ Close Details").size(14.0)).clicked() {
                            self.selected_block = None;
                        }
                    }
                    ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
                    ui.label(format!("Blocks: {}", self.blocks.len()));
                });
            });
            
            ui.separator();
            ui.add_space(5.0);
            
            // Instructions
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("💡 Drag to pan • Scroll to zoom • Click block to view details")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(156, 163, 175)));
            });
            
            ui.add_space(10.0);

            // Scrollable canvas area
            let available_rect = ui.available_rect_before_wrap();
            let canvas_response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
            
            // Handle zoom with mouse wheel
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_delta = scroll_delta * 0.001;
                let old_zoom = self.zoom;
                self.zoom = (self.zoom + zoom_delta).clamp(0.3, 3.0);
                
                // Adjust offset to zoom towards mouse position
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let mouse_canvas_pos = mouse_pos - available_rect.min - self.offset;
                    let zoom_factor = self.zoom / old_zoom;
                    self.offset = self.offset - mouse_canvas_pos * (zoom_factor - 1.0);
                }
            }
            
            // Handle dragging
            if canvas_response.dragged() && !canvas_response.clicked() {
                self.offset += canvas_response.drag_delta();
                self.is_dragging = true;
            } else if canvas_response.drag_stopped() {
                self.is_dragging = false;
            }
            
            let block_width = 180.0 * self.zoom;
            let block_spacing = 100.0 * self.zoom;
            
            // Draw blocks in snake pattern (10 per row)
            for (height, block) in self.blocks.iter().enumerate() {
                let block_pos = self.get_block_position(height, available_rect);
                let block_response = self.draw_block(ui, block, height, block_pos);
                
                // Handle block click
                if block_response.clicked() {
                    if self.selected_block == Some(height) {
                        self.selected_block = None;
                    } else {
                        self.selected_block = Some(height);
                    }
                }
                
                // Draw arrows
                if height < self.blocks.len() - 1 {
                    let next_pos = self.get_block_position(height + 1, available_rect);
                    let current_row = height / 10;
                    let next_row = (height + 1) / 10;
                    
                    if current_row == next_row {
                        // Same row - horizontal arrow
                        let row_is_reversed = current_row % 2 == 1;
                        let arrow_start = if row_is_reversed {
                            egui::Pos2::new(block_pos.x, block_pos.y + 100.0 * self.zoom)
                        } else {
                            egui::Pos2::new(block_pos.x + block_width, block_pos.y + 100.0 * self.zoom)
                        };
                        let arrow_end = if row_is_reversed {
                            egui::Pos2::new(next_pos.x + block_width, next_pos.y + 100.0 * self.zoom)
                        } else {
                            egui::Pos2::new(next_pos.x, next_pos.y + 100.0 * self.zoom)
                        };
                        self.draw_horizontal_arrow(ui, arrow_start, arrow_end);
                    } else {
                        // New row - wrap arrow
                        let arrow_start = egui::Pos2::new(block_pos.x + block_width / 2.0, block_pos.y + 200.0 * self.zoom);
                        let arrow_end = egui::Pos2::new(next_pos.x + block_width / 2.0, next_pos.y);
                        self.draw_wrap_arrow(ui, arrow_start, arrow_end);
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}