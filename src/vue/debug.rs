use crate::EmulatorApp;
use crate::hardware::memory::MemoryMap;
use crate::interpreter::disassembler::Operation;

pub fn show(
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    app: &mut EmulatorApp,
    operation: &Operation,
) {
    egui::SidePanel::left("memory_panel")
        .resizable(true) // Allow resizing the panel
        .show(ctx, |ui| {
            show_mem_map(ui, &mut app.mem_map);
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("CPU State");
        ui.label(format!("Registers: {:X?}", app.cpu));
        ui.label(format!("Next operation: {:?}", operation));

        if ui.button("Step").clicked() {
            app.step_flag = true;
        }
    });
}

fn show_mem_map(ui: &mut egui::Ui, mem_map: &mut MemoryMap) {
    use egui_extras::{Column, TableBuilder};
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto())
        .columns(Column::auto(), 16)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.centered_and_justified(|ui| ui.heading(""));
            });
            for x in 0..16 {
                header.col(|ui| {
                    ui.centered_and_justified(|ui| ui.heading(format!("{:X}", x)));
                });
            }
        })
        .body(|body| {
            body.rows(30.0, mem_map.size() / 16, |mut row| {
                let y = row.index();
                row.col(|ui| {
                    ui.centered_and_justified(|ui| ui.heading(format!("{:04X}", y * 16)));
                });
                for x in 0..16 {
                    row.col(|ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(format!("{:02X}", mem_map.read_byte(x + y * 16).unwrap()))
                        });
                    });
                }
            });
        });
}
