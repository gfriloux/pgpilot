use iced::{
  font,
  widget::{column, container, row, text, Column},
  Background, Border, Element, Font, Length,
};

use crate::app::Message;
use crate::gpg::{CheckStatus, HealthCheck};
use crate::i18n::Strings;
use crate::ui::{common, theme};

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
    text(theme::flavor(
      s.health_diagnostics_title(),
      "Report to the Commissariat",
    ))
    .size(22)
    .font(theme::flavor_title_font()),
    container(text(s.health_diagnostics_desc()).size(13),).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::text_secondary()),
        ..Default::default()
      }
    }),
  ]
  .spacing(6);

  if loading {
    let loading_content = column![
      title_section,
      container(text(s.health_checking()).size(13)).style(|_: &iced::Theme| {
        container::Style {
          text_color: Some(theme::text_muted()),
          ..Default::default()
        }
      }),
    ]
    .spacing(24);
    return common::page_layout(common::card_medium(loading_content));
  }

  // Group checks by category in order.
  // Category keys must match the strings stored in HealthCheck.category by the GPG layer.
  let categories: [(&str, &str); 3] = [
    ("Installation", s.health_category_installation()),
    ("Agent GPG", s.health_category_agent()),
    ("Sécurité", s.health_category_security()),
  ];

  let sections: Vec<Element<Message>> = categories
    .iter()
    .filter_map(|(cat_key, cat_label)| {
      let cat_checks: Vec<&HealthCheck> =
        checks.iter().filter(|c| c.category == *cat_key).collect();
      if cat_checks.is_empty() {
        return None;
      }

      let header = text(*cat_label).size(13).font(bold);

      let rows: Vec<Element<Message>> = cat_checks
        .iter()
        .map(|check| {
          let (icon, icon_color) = match check.status {
            CheckStatus::Ok => ("\u{f058}", theme::success()),
            CheckStatus::Info => ("\u{f05a}", theme::accent()),
            CheckStatus::Warning => ("\u{f071}", theme::peach()),
            CheckStatus::Error => ("\u{f057}", theme::error()),
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
                  background: Some(Background::Color(theme::header_bg())),
                  text_color: Some(theme::text_secondary()),
                  border: Border {
                    color: theme::border(),
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
                text_color: Some(theme::text_muted()),
                ..Default::default()
              })
              .into(),
          );

          if let Some(fix) = check.fix {
            content.push(
              container(text(fix).size(11).font(mono))
                .padding([6, 10])
                .style(|_: &iced::Theme| container::Style {
                  background: Some(Background::Color(theme::header_bg())),
                  text_color: Some(theme::text_secondary()),
                  border: Border {
                    color: theme::border(),
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
                color: theme::border(),
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

  let card_content =
    column![title_section, Column::with_children(sections).spacing(24),].spacing(24);

  common::page_layout(common::card_medium(card_content))
}
