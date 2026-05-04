use iced::{
  font,
  widget::{column, container, row, scrollable, text, Column},
  Background, Border, Element, Font, Length,
};

use crate::app::Message;
use crate::gpg::{CheckStatus, HealthCheck};
use crate::i18n::Strings;
use crate::ui::theme;

pub fn view<'a>(
  checks: &'a [HealthCheck],
  loading: bool,
  s: &'static dyn Strings,
) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };
  let mono = Font {
    family: font::Family::Monospace,
    ..Font::DEFAULT
  };

  let title_section = column![
    text(s.health_diagnostics_title()).size(22).font(bold),
    container(text(s.health_diagnostics_desc()).size(13),).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::TEXT_SECONDARY),
        ..Default::default()
      }
    }),
  ]
  .spacing(6);

  if loading {
    return scrollable(
      container(
        column![
          title_section,
          container(text(s.health_checking()).size(13)).style(|_: &iced::Theme| {
            container::Style {
              text_color: Some(theme::TEXT_MUTED),
              ..Default::default()
            }
          }),
        ]
        .spacing(24),
      )
      .padding(32)
      .width(560)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::CARD_BG)),
        border: Border {
          color: theme::BORDER,
          width: 1.0,
          radius: 12.0.into(),
        },
        ..Default::default()
      }),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into();
  }

  // Group checks by category in order
  let categories = ["Installation", "Agent GPG", "Sécurité"];

  let sections: Vec<Element<Message>> = categories
    .iter()
    .filter_map(|cat| {
      let cat_checks: Vec<&HealthCheck> = checks.iter().filter(|c| c.category == *cat).collect();
      if cat_checks.is_empty() {
        return None;
      }

      let header = text(*cat).size(13).font(bold);

      let rows: Vec<Element<Message>> = cat_checks
        .iter()
        .map(|check| {
          let (icon, icon_color) = match check.status {
            CheckStatus::Ok => ("\u{f058}", theme::SUCCESS),
            CheckStatus::Info => ("\u{f05a}", theme::ACCENT),
            CheckStatus::Warning => ("\u{f071}", theme::PEACH),
            CheckStatus::Error => ("\u{f057}", theme::ERROR),
          };

          let mut content: Vec<Element<Message>> = vec![row![
            text(icon).font(theme::ICONS).size(13).color(icon_color),
            text(check.name).size(13).font(bold).width(Length::Fill),
          ]
          .spacing(8)
          .into()];

          if let Some(ref val) = check.current_value {
            content.push(
              container(text(val.as_str()).size(11).font(mono))
                .padding([2, 6])
                .style(|_: &iced::Theme| container::Style {
                  background: Some(Background::Color(theme::HEADER_BG)),
                  text_color: Some(theme::TEXT_SECONDARY),
                  border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                  },
                  ..Default::default()
                })
                .into(),
            );
          }

          content.push(
            container(text(check.explanation).size(11))
              .style(|_: &iced::Theme| container::Style {
                text_color: Some(theme::TEXT_MUTED),
                ..Default::default()
              })
              .into(),
          );

          if let Some(fix) = check.fix {
            content.push(
              container(text(fix).size(11).font(mono))
                .padding([6, 10])
                .style(|_: &iced::Theme| container::Style {
                  background: Some(Background::Color(theme::HEADER_BG)),
                  text_color: Some(theme::TEXT_SECONDARY),
                  border: Border {
                    color: theme::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                  },
                  ..Default::default()
                })
                .into(),
            );
          }

          container(Column::with_children(content).spacing(6))
            .padding([10, 12])
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
              border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 6.0.into(),
              },
              ..Default::default()
            })
            .into()
        })
        .collect();

      Some(
        column![header, Column::with_children(rows).spacing(8)]
          .spacing(10)
          .into(),
      )
    })
    .collect();

  let card =
    container(column![title_section, Column::with_children(sections).spacing(24),].spacing(24))
      .padding(32)
      .width(560)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::CARD_BG)),
        border: Border {
          color: theme::BORDER,
          width: 1.0,
          radius: 12.0.into(),
        },
        text_color: Some(theme::TEXT_STRONG),
        ..Default::default()
      });

  container(
    scrollable(
      container(card)
        .center_x(Length::Fill)
        .padding([24, 0])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::SIDEBAR_BG)),
    ..Default::default()
  })
  .into()
}
