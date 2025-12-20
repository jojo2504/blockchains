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
    selected_transaction: Option<(usize, usize)>, // (block_index, tx_index)
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
            selected_transaction: None,
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
        
        let x_col = if row % 2 == 1 {
            9 - col
        } else {
            col
        };
        
        let x = available_rect.min.x + self.offset.x + (x_col as f32 * total_block_width);
        let y = available_rect.min.y + 100.0 + self.offset.y + (row as f32 * row_height);
        
        egui::Pos2::new(x, y)
    }

    fn draw_block(&self, ui: &mut egui::Ui, block: &Block, height: usize, pos: egui::Pos2) -> egui::Response {
        let block_size = egui::Vec2::new(180.0 * self.zoom, 220.0 * self.zoom);
        let rect = egui::Rect::from_min_size(pos, block_size);
        
        let response = ui.allocate_rect(rect, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            let is_selected = self.selected_block == Some(height);
            let base_color = if is_selected {
                egui::Color32::from_rgb(59, 130, 246)
            } else if response.hovered() {
                egui::Color32::from_rgb(51, 65, 85)
            } else {
                egui::Color32::from_rgb(30, 41, 59)
            };
            
            let stroke_color = if is_selected {
                egui::Color32::from_rgb(96, 165, 250)
            } else {
                egui::Color32::from_rgb(71, 85, 105)
            };
            
            let stroke_width = if is_selected { 3.0 * self.zoom } else { 2.0 * self.zoom };
            
            // Shadow
            let shadow_offset = 4.0 * self.zoom;
            let shadow_rect = rect.translate(egui::Vec2::new(shadow_offset, shadow_offset));
            painter.rect(
                shadow_rect.shrink(5.0 * self.zoom),
                8.0 * self.zoom,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 40),
                egui::Stroke::NONE,
                egui::epaint::StrokeKind::Outside,
            );
            
            // Main block
            painter.rect(
                rect.shrink(5.0 * self.zoom),
                8.0 * self.zoom,
                base_color,
                egui::Stroke::new(stroke_width, stroke_color),
                egui::epaint::StrokeKind::Outside,
            );
            
            // Top accent
            let top_rect = egui::Rect::from_min_size(
                rect.min + egui::Vec2::new(5.0 * self.zoom, 5.0 * self.zoom),
                egui::Vec2::new(rect.width() - 10.0 * self.zoom, 4.0 * self.zoom)
            );
            painter.rect_filled(top_rect, 2.0 * self.zoom, egui::Color32::from_rgb(96, 165, 250));
            
            let text_rect = rect.shrink(15.0 * self.zoom);
            let content_start_y = text_rect.min.y + 15.0 * self.zoom;
            
            // Block icon
            painter.text(
                egui::Pos2::new(rect.center().x, content_start_y),
                egui::Align2::CENTER_TOP,
                "■",
                egui::FontId::proportional(32.0 * self.zoom),
                egui::Color32::from_rgb(96, 165, 250),
            );
            
            painter.text(
                egui::Pos2::new(rect.center().x, content_start_y + 35.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                format!("Block #{}", height),
                egui::FontId::proportional(14.0 * self.zoom),
                egui::Color32::from_rgb(226, 232, 240),
            );
            
            // Hash preview
            let hash_hex = block.hash.0.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            let hash_display = format!("{}...", &hash_hex[..8]);
            
            painter.text(
                egui::Pos2::new(rect.center().x, content_start_y + 60.0 * self.zoom),
                egui::Align2::CENTER_TOP,
                hash_display,
                egui::FontId::monospace(10.0 * self.zoom),
                egui::Color32::from_rgb(148, 163, 184),
            );
            
            // Stats
            let stats_y = content_start_y + 85.0 * self.zoom;
            
            painter.text(
                egui::Pos2::new(rect.min.x + 20.0 * self.zoom, stats_y),
                egui::Align2::LEFT_TOP,
                "TX",
                egui::FontId::proportional(9.0 * self.zoom),
                egui::Color32::from_rgb(100, 116, 139),
            );
            painter.text(
                egui::Pos2::new(rect.min.x + 20.0 * self.zoom, stats_y + 14.0 * self.zoom),
                egui::Align2::LEFT_TOP,
                format!("{}", block.transactions.len()),
                egui::FontId::proportional(16.0 * self.zoom),
                egui::Color32::from_rgb(34, 197, 94),
            );
            
            painter.text(
                egui::Pos2::new(rect.max.x - 20.0 * self.zoom, stats_y),
                egui::Align2::RIGHT_TOP,
                "NONCE",
                egui::FontId::proportional(9.0 * self.zoom),
                egui::Color32::from_rgb(100, 116, 139),
            );
            let nonce_display = if block.nonce > 9999 {
                format!("{}k", block.nonce / 1000)
            } else {
                format!("{}", block.nonce)
            };
            painter.text(
                egui::Pos2::new(rect.max.x - 20.0 * self.zoom, stats_y + 14.0 * self.zoom),
                egui::Align2::RIGHT_TOP,
                nonce_display,
                egui::FontId::proportional(16.0 * self.zoom),
                egui::Color32::from_rgb(251, 146, 60),
            );
            
            if let Some(timestamp) = block.timestamp {
                painter.text(
                    egui::Pos2::new(rect.center().x, rect.max.y - 20.0 * self.zoom),
                    egui::Align2::CENTER_BOTTOM,
                    timestamp.format("%H:%M:%S").to_string(),
                    egui::FontId::proportional(9.0 * self.zoom),
                    egui::Color32::from_rgb(100, 116, 139),
                );
            }
        }
        
        response
    }

    fn draw_horizontal_arrow(&self, ui: &mut egui::Ui, from_pos: egui::Pos2, to_pos: egui::Pos2) {
        let painter = ui.painter();
        painter.line_segment(
            [from_pos, to_pos],
            egui::Stroke::new(3.0 * self.zoom, egui::Color32::from_rgb(71, 85, 105)),
        );
        
        let arrow_size = 10.0 * self.zoom;
        let direction = (to_pos - from_pos).normalized();
        let perpendicular = egui::Vec2::new(-direction.y, direction.x);
        
        let arrow_tip = to_pos;
        let arrow_left = arrow_tip - direction * arrow_size + perpendicular * (arrow_size / 2.0);
        let arrow_right = arrow_tip - direction * arrow_size - perpendicular * (arrow_size / 2.0);
        
        painter.add(egui::Shape::convex_polygon(
            vec![arrow_tip, arrow_left, arrow_right],
            egui::Color32::from_rgb(71, 85, 105),
            egui::Stroke::NONE,
        ));
    }

    fn draw_wrap_arrow(&self, ui: &mut egui::Ui, from_pos: egui::Pos2, to_pos: egui::Pos2) {
        let painter = ui.painter();
        let stroke = egui::Stroke::new(3.0 * self.zoom, egui::Color32::from_rgb(71, 85, 105));
        
        let mid_y = from_pos.y + 80.0 * self.zoom;
        let mid_point1 = egui::Pos2::new(from_pos.x, mid_y);
        let mid_point2 = egui::Pos2::new(to_pos.x, mid_y);
        
        painter.line_segment([from_pos, mid_point1], stroke);
        painter.line_segment([mid_point1, mid_point2], stroke);
        painter.line_segment([mid_point2, to_pos], stroke);
        
        let arrow_size = 10.0 * self.zoom;
        let arrow_tip = to_pos;
        let arrow_left = arrow_tip + egui::Vec2::new(-arrow_size / 2.0, arrow_size);
        let arrow_right = arrow_tip + egui::Vec2::new(arrow_size / 2.0, arrow_size);
        
        painter.add(egui::Shape::convex_polygon(
            vec![arrow_tip, arrow_left, arrow_right],
            egui::Color32::from_rgb(71, 85, 105),
            egui::Stroke::NONE,
        ));
    }

    fn draw_transaction_details(&self, ui: &mut egui::Ui, tx: &crate::types::transaction::Transaction, block_height: usize, tx_index: usize) -> bool {
        let mut close_clicked = false;
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(format!("Transaction #{}", tx_index + 1))
                    .size(28.0)
                    .color(egui::Color32::from_rgb(34, 197, 94)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("X Close").size(14.0)).clicked() {
                        close_clicked = true;
                    }
                });
            });
            
            ui.label(egui::RichText::new(format!("From Block #{}", block_height))
                .size(12.0)
                .color(egui::Color32::from_rgb(100, 116, 139)));
            
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // TXID
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Transaction ID")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(148, 163, 184))
                        .strong());
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(&tx.txid)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                });

            ui.add_space(15.0);

            // Addresses
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("ADDRESSES")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(96, 165, 250))
                        .strong());
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("From Address")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.add_space(4.0);
                    let from_hex = tx.from.0.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(from_hex)
                            .size(12.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });

                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("To Address")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(148, 163, 184)));
                    ui.add_space(4.0);
                    match &tx.to {
                        Some(addr) => {
                            let to_hex = addr.0.iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>();
                            ui.horizontal_wrapped(|ui| {
                                ui.monospace(egui::RichText::new(to_hex)
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(226, 232, 240)));
                            });
                        }
                        None => {
                            ui.label(egui::RichText::new("N/A (Message-only transaction)")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(148, 163, 184))
                                .italics());
                        }
                    }
                });

            ui.add_space(15.0);

            // Transaction Details
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("TRANSACTION DETAILS")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(96, 165, 250))
                        .strong());
                    ui.add_space(10.0);

                    ui.columns(2, |columns| {
                        if let Some(amount) = tx.amount {
                            columns[0].label(egui::RichText::new("Amount")
                                .color(egui::Color32::from_rgb(148, 163, 184)));
                            columns[1].label(egui::RichText::new(format!("{}", amount))
                                .color(egui::Color32::from_rgb(34, 197, 94))
                                .strong());
                        }

                        columns[0].label(egui::RichText::new("Fee")
                            .color(egui::Color32::from_rgb(148, 163, 184)));
                        columns[1].label(egui::RichText::new(format!("{}", tx.fee))
                            .color(egui::Color32::from_rgb(251, 146, 60)));

                        columns[0].label(egui::RichText::new("Nonce")
                            .color(egui::Color32::from_rgb(148, 163, 184)));
                        columns[1].label(egui::RichText::new(format!("{}", tx.nonce))
                            .color(egui::Color32::from_rgb(226, 232, 240)));

                        columns[0].label(egui::RichText::new("Size")
                            .color(egui::Color32::from_rgb(148, 163, 184)));
                        columns[1].label(egui::RichText::new(format!("{} bytes", tx.size))
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                });

            // Message
            if let Some(msg) = &tx.message {
                ui.add_space(15.0);
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(67, 56, 202))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(99, 102, 241)))
                    .corner_radius(8.0)
                    .inner_margin(15.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("MESSAGE")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(199, 210, 254))
                            .strong());
                        ui.add_space(8.0);
                        let msg_display = String::from_utf8(msg.clone())
                            .unwrap_or_else(|_| {
                                format!("0x{}", msg.iter().map(|b| format!("{:02x}", b)).collect::<String>())
                            });
                        ui.label(egui::RichText::new(msg_display)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(224, 231, 255)));
                    });
            }

            // Signature
            ui.add_space(15.0);
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("SIGNATURE")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(96, 165, 250))
                        .strong());
                    ui.add_space(6.0);
                    let sig_hex = tx.signature.0.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(egui::RichText::new(sig_hex)
                            .size(11.0)
                            .color(egui::Color32::from_rgb(203, 213, 225)));
                    });
                });

            ui.add_space(20.0);
        });
        close_clicked
    }

    fn draw_block_details(&self, ui: &mut egui::Ui, block: &Block, height: usize) -> (bool, Option<usize>) {
        let mut close_clicked = false;
        let mut view_tx_clicked: Option<usize> = None;
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(format!("Block #{}", height))
                    .size(32.0)
                    .color(egui::Color32::from_rgb(147, 197, 253)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("X Close").size(16.0)).clicked() {
                        close_clicked = true;
                    }
                });
            });
            
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // Header
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(66, 153, 225)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("BLOCK HEADER")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(147, 197, 253))
                        .strong());
                    
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Hash:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        let hash_hex = block.hash.0.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        ui.monospace(egui::RichText::new(hash_hex)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Previous Hash:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        let prev_hash_display = match &block.previous_hash {
                            Some(hash) => hash.0.iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>(),
                            None => "None (Genesis Block)".to_string(),
                        };
                        ui.monospace(egui::RichText::new(prev_hash_display)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Merkle Root:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        let merkle_hex = block.merkle_root.0.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        ui.monospace(egui::RichText::new(merkle_hex)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(226, 232, 240)));
                    });
                    
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Nonce:")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(148, 163, 184))
                            .strong());
                        ui.label(egui::RichText::new(format!("{}", block.nonce))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(251, 191, 36)));
                    });

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

            // Transactions
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(66, 153, 225)))
                .corner_radius(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("TRANSACTIONS ({})", block.transactions.len()))
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
                            let tx_response = egui::Frame::new()
                                .fill(egui::Color32::from_rgb(15, 23, 42))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 65, 85)))
                                .corner_radius(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("TX #{}", i + 1))
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(34, 197, 94))
                                            .strong());
                                        
                                        let txid_short = if tx.txid.len() > 16 {
                                            format!("{}...", &tx.txid[..16])
                                        } else {
                                            tx.txid.clone()
                                        };
                                        ui.label(egui::RichText::new(txid_short)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(148, 163, 184)));
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button(egui::RichText::new("View Details >>").size(12.0)).clicked() {
                                                view_tx_clicked = Some(i);
                                            }
                                            if let Some(amount) = tx.amount {
                                                ui.label(egui::RichText::new(format!("{}", amount))
                                                    .size(13.0)
                                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                                            }
                                        });
                                    });
                                }).response;
                            
                            ui.add_space(8.0);
                        }
                    }
                });

            ui.add_space(20.0);
        });
        (close_clicked, view_tx_clicked)
    }
}

impl eframe::App for BlockchainViewer {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Drain events
        loop {
            match self.rx.try_recv() {
                Ok(NodeEvent::NewBlock(block)) => {
                    self.blocks.push(block);
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                    eprintln!("Warning: Viewer lagged! Skipped {} blocks", skipped);
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    eprintln!("Event channel closed");
                    break;
                }
            }
        }

        // Transaction details panel
        let mut close_tx_panel = false;
        if let Some((block_idx, tx_idx)) = self.selected_transaction {
            egui::SidePanel::right("tx_details")
                .min_width(500.0)
                .max_width(700.0)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(block) = self.blocks.get(block_idx) {
                        if let Some(tx) = block.transactions.get(tx_idx) {
                            close_tx_panel = self.draw_transaction_details(ui, tx, block_idx, tx_idx);
                        }
                    }
                });
        }
        
        if close_tx_panel {
            self.selected_transaction = None;
        }

        // Block details panel
        let mut close_block_panel = false;
        let mut open_tx_details: Option<usize> = None;
        if let Some(selected_height) = self.selected_block {
            egui::SidePanel::right("block_details")
                .min_width(500.0)
                .max_width(700.0)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(block) = self.blocks.get(selected_height) {
                        let (close_clicked, tx_clicked) = self.draw_block_details(ui, block, selected_height);
                        close_block_panel = close_clicked;
                        open_tx_details = tx_clicked;
                    }
                });
        }
        
        if close_block_panel {
            self.selected_block = None;
            self.selected_transaction = None;
        }
        
        if let Some(tx_idx) = open_tx_details {
            if let Some(block_idx) = self.selected_block {
                self.selected_transaction = Some((block_idx, tx_idx));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar
            ui.horizontal(|ui| {
                ui.heading("Blockchain Explorer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.selected_transaction.is_some() {
                        if ui.button(egui::RichText::new("X Close TX").size(14.0)).clicked() {
                            self.selected_transaction = None;
                        }
                    }
                    if self.selected_block.is_some() {
                        if ui.button(egui::RichText::new("X Close Block").size(14.0)).clicked() {
                            self.selected_block = None;
                            self.selected_transaction = None;
                        }
                    }
                    ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
                    ui.label(format!("Blocks: {}", self.blocks.len()));
                });
            });
            
            ui.separator();
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Drag to pan - Scroll to zoom - Click block for details")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(156, 163, 175)));
            });
            
            ui.add_space(10.0);

            let available_rect = ui.available_rect_before_wrap();
            let canvas_response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
            
            // Zoom
            if canvas_response.hovered() {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    let zoom_delta = scroll_delta * 0.001;
                    let old_zoom = self.zoom;
                    self.zoom = (self.zoom + zoom_delta).clamp(0.3, 3.0);
                    
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let mouse_canvas_pos = mouse_pos - available_rect.min - self.offset;
                        let zoom_factor = self.zoom / old_zoom;
                        self.offset = self.offset - mouse_canvas_pos * (zoom_factor - 1.0);
                    }
                }
            }
            
            // Dragging
            if canvas_response.dragged() && !canvas_response.clicked() {
                self.offset += canvas_response.drag_delta();
                self.is_dragging = true;
            } else if canvas_response.drag_stopped() {
                self.is_dragging = false;
            }
            
            let block_width = 180.0 * self.zoom;
            
            // Draw blocks
            for (height, block) in self.blocks.iter().enumerate() {
                let block_pos = self.get_block_position(height, available_rect);
                let block_response = self.draw_block(ui, block, height, block_pos);
                
                if block_response.clicked() {
                    if self.selected_block == Some(height) {
                        self.selected_block = None;
                        self.selected_transaction = None;
                    } else {
                        self.selected_block = Some(height);
                        self.selected_transaction = None;
                    }
                }
                
                // Arrows
                if height < self.blocks.len() - 1 {
                    let next_pos = self.get_block_position(height + 1, available_rect);
                    let current_row = height / 10;
                    let next_row = (height + 1) / 10;
                    
                    if current_row == next_row {
                        let row_is_reversed = current_row % 2 == 1;
                        let arrow_start = if row_is_reversed {
                            egui::Pos2::new(block_pos.x, block_pos.y + 110.0 * self.zoom)
                        } else {
                            egui::Pos2::new(block_pos.x + block_width, block_pos.y + 110.0 * self.zoom)
                        };
                        let arrow_end = if row_is_reversed {
                            egui::Pos2::new(next_pos.x + block_width, next_pos.y + 110.0 * self.zoom)
                        } else {
                            egui::Pos2::new(next_pos.x, next_pos.y + 110.0 * self.zoom)
                        };
                        self.draw_horizontal_arrow(ui, arrow_start, arrow_end);
                    } else {
                        let arrow_start = egui::Pos2::new(block_pos.x + block_width / 2.0, block_pos.y + 220.0 * self.zoom);
                        let arrow_end = egui::Pos2::new(next_pos.x + block_width / 2.0, next_pos.y);
                        self.draw_wrap_arrow(ui, arrow_start, arrow_end);
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}