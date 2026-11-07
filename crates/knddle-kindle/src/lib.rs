//! Catalog of Kindle e-readers supported by Knddle (5th generation onwards)
//! and the rules that pick the best format and image size for each one.
//!
//! Screen data comes from Amazon's published specifications. Each entry
//! must be confirmed on real hardware before the 1.0 release.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub ppi: u32,
}

/// How the device appears to the computer over USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    /// A regular drive with a `documents/` folder.
    MassStorage,
    /// Media Transfer Protocol: no drive letter or mount point.
    Mtp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BookFormat {
    Kfx,
    Azw3,
    Mobi,
    Epub,
}

impl BookFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Kfx => "kfx",
            Self::Azw3 => "azw3",
            Self::Mobi => "mobi",
            Self::Epub => "epub",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct KindleModel {
    pub id: &'static str,
    pub name: &'static str,
    pub year: u16,
    pub screen: Screen,
    pub connection: Connection,
    /// Supports KFX / Enhanced Typesetting when sideloaded.
    pub kfx: bool,
}

/// Which KFX generator is available on this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KfxSupport {
    Unavailable,
    Available,
}

impl KindleModel {
    /// Best format to copy over USB. Every model in the catalog reads AZW3,
    /// so it is the fallback whenever KFX cannot be produced or read.
    pub fn usb_format(&self, kfx: KfxSupport) -> BookFormat {
        if self.kfx && kfx == KfxSupport::Available {
            BookFormat::Kfx
        } else {
            BookFormat::Azw3
        }
    }

    /// Largest image box that fits the screen in portrait orientation.
    pub fn image_box(&self) -> (u32, u32) {
        (self.screen.width, self.screen.height)
    }

    /// Cover thumbnail height shown in the Kindle library grid.
    pub fn thumbnail_height(&self) -> u32 {
        self.screen.height / THUMBNAIL_FRACTION_OF_SCREEN
    }
}

/// Email delivery always sends EPUB: Amazon converts it to KFX itself.
pub const EMAIL_FORMAT: BookFormat = BookFormat::Epub;

const THUMBNAIL_FRACTION_OF_SCREEN: u32 = 4;

const fn screen(width: u32, height: u32, ppi: u32) -> Screen {
    Screen { width, height, ppi }
}

const fn model(
    id: &'static str,
    name: &'static str,
    year: u16,
    screen: Screen,
    connection: Connection,
    kfx: bool,
) -> KindleModel {
    KindleModel {
        id,
        name,
        year,
        screen,
        connection,
        kfx,
    }
}

use Connection::{MassStorage, Mtp};

pub const CATALOG: &[KindleModel] = &[
    model(
        "kindle-5",
        "Kindle (5ª gen)",
        2012,
        screen(600, 800, 167),
        MassStorage,
        false,
    ),
    model(
        "paperwhite-1",
        "Kindle Paperwhite (1ª gen)",
        2012,
        screen(758, 1024, 212),
        MassStorage,
        false,
    ),
    model(
        "paperwhite-2",
        "Kindle Paperwhite (2ª gen)",
        2013,
        screen(758, 1024, 212),
        MassStorage,
        true,
    ),
    model(
        "kindle-7",
        "Kindle (7ª gen)",
        2014,
        screen(600, 800, 167),
        MassStorage,
        true,
    ),
    model(
        "voyage",
        "Kindle Voyage",
        2014,
        screen(1072, 1448, 300),
        MassStorage,
        true,
    ),
    model(
        "paperwhite-3",
        "Kindle Paperwhite (3ª gen)",
        2015,
        screen(1072, 1448, 300),
        MassStorage,
        true,
    ),
    model(
        "oasis-1",
        "Kindle Oasis (1ª gen)",
        2016,
        screen(1072, 1448, 300),
        MassStorage,
        true,
    ),
    model(
        "kindle-8",
        "Kindle (8ª gen)",
        2016,
        screen(600, 800, 167),
        MassStorage,
        true,
    ),
    model(
        "oasis-2",
        "Kindle Oasis (2ª gen)",
        2017,
        screen(1264, 1680, 300),
        MassStorage,
        true,
    ),
    model(
        "paperwhite-4",
        "Kindle Paperwhite (4ª gen)",
        2018,
        screen(1072, 1448, 300),
        MassStorage,
        true,
    ),
    model(
        "kindle-10",
        "Kindle (10ª gen)",
        2019,
        screen(600, 800, 167),
        MassStorage,
        true,
    ),
    model(
        "oasis-3",
        "Kindle Oasis (3ª gen)",
        2019,
        screen(1264, 1680, 300),
        MassStorage,
        true,
    ),
    model(
        "paperwhite-5",
        "Kindle Paperwhite (5ª gen)",
        2021,
        screen(1236, 1648, 300),
        MassStorage,
        true,
    ),
    model(
        "kindle-11",
        "Kindle (11ª gen)",
        2022,
        screen(1072, 1448, 300),
        MassStorage,
        true,
    ),
    model(
        "scribe-1",
        "Kindle Scribe (1ª gen)",
        2022,
        screen(1860, 2480, 300),
        MassStorage,
        true,
    ),
    model(
        "kindle-12",
        "Kindle (2024)",
        2024,
        screen(1072, 1448, 300),
        Mtp,
        true,
    ),
    model(
        "paperwhite-6",
        "Kindle Paperwhite (2024)",
        2024,
        screen(1264, 1680, 300),
        Mtp,
        true,
    ),
    model(
        "colorsoft",
        "Kindle Colorsoft",
        2024,
        screen(1264, 1680, 300),
        Mtp,
        true,
    ),
    model(
        "scribe-2",
        "Kindle Scribe (2024)",
        2024,
        screen(1860, 2480, 300),
        Mtp,
        true,
    ),
];

pub fn find(id: &str) -> Option<&'static KindleModel> {
    CATALOG.iter().find(|model| model.id == id)
}
