#[test]
fn card_status_without_card_does_not_panic() {
  let result = pgpilot::gpg::card::card_status();
  // Sans carte physique : None — ne doit pas paniquer
  let _ = result;
}
