pub mod components;
pub mod modals;
pub mod screens;

use crate::models::ConfigSyncField;
use crate::models::field_accessor::FieldType;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum ClickAction {
    StartupToday,
    StartupLogs,
    StartupAddDate,
    OpenStatistics,
    OpenCloudSync,
    Quit,
    BackToStartup,
    OpenLog(usize),
    EditField(FieldType),
    AddFood,
    SelectFood(usize),
    AddSokay,
    SelectSokay(usize),
    StrengthMobility,
    Notes,
    FocusConfigField(ConfigSyncField),
    ToggleConfigSync,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClickTarget {
    pub area: Rect,
    pub action: ClickAction,
}

impl ClickTarget {
    pub fn new(area: Rect, action: ClickAction) -> Self {
        Self { area, action }
    }
}

pub fn hit_test(targets: &[ClickTarget], column: u16, row: u16) -> Option<ClickAction> {
    let position = Position::new(column, row);
    targets
        .iter()
        .rev()
        .find(|target| target.area.contains(position))
        .map(|target| target.action.clone())
}

pub fn left_click_position(mouse: MouseEvent) -> Option<(u16, u16)> {
    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
        Some((mouse.column, mouse.row))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn hit_test_uses_half_open_rectangle_boundaries() {
        let targets = vec![ClickTarget::new(
            Rect::new(2, 3, 4, 2),
            ClickAction::OpenStatistics,
        )];

        assert_eq!(hit_test(&targets, 2, 3), Some(ClickAction::OpenStatistics));
        assert_eq!(hit_test(&targets, 5, 4), Some(ClickAction::OpenStatistics));
        assert_eq!(hit_test(&targets, 6, 4), None);
        assert_eq!(hit_test(&targets, 5, 5), None);
    }

    #[test]
    fn hit_test_prefers_the_last_rendered_target() {
        let targets = vec![
            ClickTarget::new(Rect::new(0, 0, 10, 10), ClickAction::AddFood),
            ClickTarget::new(Rect::new(2, 2, 4, 4), ClickAction::Notes),
        ];

        assert_eq!(hit_test(&targets, 3, 3), Some(ClickAction::Notes));
    }

    #[test]
    fn only_left_button_down_is_a_click() {
        let mouse = |kind| MouseEvent {
            kind,
            column: 7,
            row: 9,
            modifiers: KeyModifiers::NONE,
        };

        assert_eq!(
            left_click_position(mouse(MouseEventKind::Down(MouseButton::Left))),
            Some((7, 9))
        );
        assert_eq!(
            left_click_position(mouse(MouseEventKind::Down(MouseButton::Right))),
            None
        );
        assert_eq!(left_click_position(mouse(MouseEventKind::ScrollDown)), None);
        assert_eq!(left_click_position(mouse(MouseEventKind::Moved)), None);
    }
}
