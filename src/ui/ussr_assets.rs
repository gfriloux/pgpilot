use std::sync::OnceLock;

use iced::widget::{image, svg};

// ── Bannières propagandistes ──────────────────────────────────────────────
//
// Statics mémoïsés : le handle est créé une seule fois pour éviter de créer
// un nouvel Arc à chaque rendu, ce qui provoquerait un flash visuel (iced
// détecte le changement de handle et re-décode l'image).

static B12: OnceLock<image::Handle> = OnceLock::new();
static B16: OnceLock<image::Handle> = OnceLock::new();
static B17: OnceLock<image::Handle> = OnceLock::new();
static B18: OnceLock<image::Handle> = OnceLock::new();
static B19: OnceLock<image::Handle> = OnceLock::new();
static B20: OnceLock<image::Handle> = OnceLock::new();
static B23: OnceLock<image::Handle> = OnceLock::new();
static B24: OnceLock<image::Handle> = OnceLock::new();
static B25: OnceLock<image::Handle> = OnceLock::new();
static B26: OnceLock<image::Handle> = OnceLock::new();
static B27: OnceLock<image::Handle> = OnceLock::new();
static B29: OnceLock<image::Handle> = OnceLock::new();

fn mk(lock: &'static OnceLock<image::Handle>, bytes: &'static [u8]) -> image::Handle {
  lock
    .get_or_init(|| image::Handle::from_bytes(bytes))
    .clone()
}

/// Retourne le handle de la bannière pour la vue donnée (mémoïsé).
///
/// Numéros assignés par vue :
/// - 12 : Diagnostic
/// - 16 : Chiffrer
/// - 17 : Vérifier + Clefs Publiques (pied)
/// - 18 : Mes Clefs (liste)
/// - 19 : Déchiffrer
/// - 20 : Signer
/// - 23 : Clefs Publiques (liste)
/// - 24 : Importer
/// - 25 : Mes Clefs (pied détail)
/// - 26 : Chat (panel rooms)
/// - 27 : Créer une clef
/// - 29 : Paramètres
pub fn banner(n: u8) -> image::Handle {
  match n {
    12 => mk(&B12, include_bytes!("../../assets/banners/12.png")),
    16 => mk(&B16, include_bytes!("../../assets/banners/16.png")),
    17 => mk(&B17, include_bytes!("../../assets/banners/17.png")),
    18 => mk(&B18, include_bytes!("../../assets/banners/18.png")),
    19 => mk(&B19, include_bytes!("../../assets/banners/19.png")),
    20 => mk(&B20, include_bytes!("../../assets/banners/20.png")),
    23 => mk(&B23, include_bytes!("../../assets/banners/23.png")),
    24 => mk(&B24, include_bytes!("../../assets/banners/24.png")),
    25 => mk(&B25, include_bytes!("../../assets/banners/25.png")),
    26 => mk(&B26, include_bytes!("../../assets/banners/26.png")),
    27 => mk(&B27, include_bytes!("../../assets/banners/27.png")),
    29 => mk(&B29, include_bytes!("../../assets/banners/29.png")),
    _ => mk(&B17, include_bytes!("../../assets/banners/17.png")),
  }
}

// ── Badges SVG circulaires ────────────────────────────────────────────────

static BADGE_KS: OnceLock<svg::Handle> = OnceLock::new();
static BADGE_YK: OnceLock<svg::Handle> = OnceLock::new();
static BADGE_TF: OnceLock<svg::Handle> = OnceLock::new();
static BADGE_TM: OnceLock<svg::Handle> = OnceLock::new();
static BADGE_TU: OnceLock<svg::Handle> = OnceLock::new();

fn mk_svg(lock: &'static OnceLock<svg::Handle>, bytes: &'static [u8]) -> svg::Handle {
  lock.get_or_init(|| svg::Handle::from_memory(bytes)).clone()
}

pub fn badge_keyserver() -> svg::Handle {
  mk_svg(
    &BADGE_KS,
    include_bytes!("../../assets/badge_keyserver.svg"),
  )
}

pub fn badge_yubikey() -> svg::Handle {
  mk_svg(&BADGE_YK, include_bytes!("../../assets/badge_yubikey.svg"))
}

pub fn badge_trust_full() -> svg::Handle {
  mk_svg(
    &BADGE_TF,
    include_bytes!("../../assets/badge_trust_full.svg"),
  )
}

pub fn badge_trust_marginal() -> svg::Handle {
  mk_svg(
    &BADGE_TM,
    include_bytes!("../../assets/badge_trust_marginal.svg"),
  )
}

pub fn badge_trust_undef() -> svg::Handle {
  mk_svg(
    &BADGE_TU,
    include_bytes!("../../assets/badge_trust_undef.svg"),
  )
}

// Le séparateur étoile est implémenté via common::star_separator() en iced
// primitives (rule::horizontal + text) — pas besoin d'un SVG externe.
