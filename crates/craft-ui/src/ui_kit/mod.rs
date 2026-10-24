//! Tokenised dense card library. Import elements through `craft_ui::ui_kit`.
//! `Panel` owns card composition; rows own aligned label/control columns and retain egui focus.
//! Geometry, typography and colours come from `theme::Tokens`. See `docs/ui-kit.md`.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/ui_kit/`), MIT OR Apache-2.0. See ATTRIBUTION.md.

mod actions;
mod checks;
mod choice;
mod color;
mod confirm;
pub mod container;
mod disclosure;
mod fields;
mod file_view;
mod history;
mod layout;
mod menu;
mod metrics;
mod panel;
mod preferences;
mod preferences_small;
mod radial;
mod reports;
mod search;
mod shortcuts;
mod status;
mod table;
mod text;

pub use actions::*;
pub use checks::*;
pub use choice::*;
pub use color::*;
pub use confirm::{Confirm, Tone};
pub use container::{Container, DockEdges, Edge, Surface};
pub use disclosure::*;
pub use fields::*;
pub use file_view::{FileView, RecentRow, SampleCard};
pub use history::{HistoryChip, HistoryTransport, TREE_ROW_HEIGHT, history_marker, history_transport_width, panel_grip};
pub use layout::*;
pub use menu::{Menu, MenuItem};
pub use metrics::{ContentWidth, header_height, row_height, table_row_height};
pub use panel::{ComputeStatus, FeatureOptions, Panel, Window, collapsible_header, header, panel_header_hover};
pub use preferences::*;
pub use preferences_small::*;
pub use radial::{RADIAL_RADIUS, radial_menu};
pub use reports::*;
pub use search::{CommandSearch, SearchBadge, SearchFieldGeometry};
pub use shortcuts::*;
pub use status::*;
pub use table::*;
pub use text::*;

pub use layout::label as row_label;
