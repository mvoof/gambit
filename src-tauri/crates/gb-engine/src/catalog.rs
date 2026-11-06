//! The engines the app can install: pinned releases, hashes from their GitHub assets.

/// How a release is packed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFormat {
    Binary,
    Zip,
    TarGz,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Package {
    pub url: &'static str,
    pub bytes: u32,
    pub sha256: &'static str,
    pub format: PackageFormat,
}

/// A release may have a build for AVX2, one for any x86-64 CPU, or both.
#[derive(Debug)]
pub struct Builds {
    pub avx2: Option<Package>,
    pub baseline: Option<Package>,
}

impl Builds {
    const NONE: Self = Self {
        avx2: None,
        baseline: None,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
}

impl Platform {
    /// `None` where no catalogue build runs.
    pub fn current() -> Option<Self> {
        if !cfg!(target_arch = "x86_64") {
            None
        } else if cfg!(windows) {
            Some(Self::Windows)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else {
            None
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub fn has_avx2() -> bool {
    is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
pub fn has_avx2() -> bool {
    false
}

#[derive(Debug)]
pub struct CatalogEngine {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub licence: &'static str,
    pub homepage: &'static str,
    pub windows: Builds,
    pub linux: Builds,
}

impl CatalogEngine {
    pub fn package(&self, platform: Platform, avx2: bool) -> Option<&Package> {
        let builds = match platform {
            Platform::Windows => &self.windows,
            Platform::Linux => &self.linux,
        };
        let fastest = if avx2 { builds.avx2.as_ref() } else { None };

        fastest.or(builds.baseline.as_ref())
    }
}

const fn binary(url: &'static str, bytes: u32, sha256: &'static str) -> Option<Package> {
    Some(Package {
        url,
        bytes,
        sha256,
        format: PackageFormat::Binary,
    })
}

pub const CATALOG: [CatalogEngine; 8] = [
    CatalogEngine {
        id: "stockfish",
        name: "Stockfish",
        version: "19",
        licence: "GPL-3.0",
        homepage: "https://stockfishchess.org",
        windows: Builds {
            avx2: None,
            baseline: Some(Package {
                url: "https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-windows-x86-64-universal.zip",
                bytes: 81_431_614,
                sha256: "3c8bf1f9ea66a09350a40df4f632288285ac206d99f33ab5842c408fc30b48a7",
                format: PackageFormat::Zip,
            }),
        },
        linux: Builds {
            avx2: None,
            baseline: Some(Package {
                url: "https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-linux-x86-64-universal.tar.gz",
                bytes: 81_388_977,
                sha256: "9defc0d4e55d49c65a6d042f3e571a39fcea499ade6dbe741b53b8c65e03611f",
                format: PackageFormat::TarGz,
            }),
        },
    },
    CatalogEngine {
        id: "reckless",
        name: "Reckless",
        version: "0.9.0",
        licence: "AGPL-3.0",
        homepage: "https://github.com/codedeliveryservice/Reckless",
        windows: Builds {
            avx2: binary(
                "https://github.com/codedeliveryservice/Reckless/releases/download/v0.9.0/reckless-windows-avx2.exe",
                64_664_576,
                "b74ead5648cfa7a7a9f51d04566cf00f56dcf90dacd1252990906223bf1891b8",
            ),
            baseline: binary(
                "https://github.com/codedeliveryservice/Reckless/releases/download/v0.9.0/reckless-windows-generic.exe",
                64_625_664,
                "26f4eb1382fb1b3fff5fa599545ec38a9e6375bf48df3f3038b7cb5a7e685c34",
            ),
        },
        linux: Builds {
            avx2: binary(
                "https://github.com/codedeliveryservice/Reckless/releases/download/v0.9.0/reckless-linux-avx2",
                65_004_440,
                "09ba1634faaffec55d237a7efecfb27d5152f6f1400f24dd63af9bde00a054f6",
            ),
            baseline: binary(
                "https://github.com/codedeliveryservice/Reckless/releases/download/v0.9.0/reckless-linux-generic",
                64_964_984,
                "a165c726227593a0364e4d52d9a5a635f1b6e2481d559f043b4563957fda8685",
            ),
        },
    },
    CatalogEngine {
        id: "plentychess",
        name: "PlentyChess",
        version: "8.0.0",
        licence: "GPL-3.0",
        homepage: "https://github.com/Yoshie2000/PlentyChess",
        windows: Builds {
            avx2: binary(
                "https://github.com/Yoshie2000/PlentyChess/releases/download/b-v8.0.0/PlentyChess-8.0.0-windows-avx2.exe",
                53_943_296,
                "64a3b305da9c17eab8d628f699d41149e0013ffd6d4cc10cceb5808d98a292e2",
            ),
            baseline: binary(
                "https://github.com/Yoshie2000/PlentyChess/releases/download/b-v8.0.0/PlentyChess-8.0.0-windows-generic.exe",
                55_205_376,
                "a29fb8be1edb4bc961290bda03d06acc38a4ed35c2cf9ab235c7c8db0da38ad6",
            ),
        },
        linux: Builds {
            avx2: binary(
                "https://github.com/Yoshie2000/PlentyChess/releases/download/b-v8.0.0/PlentyChess-8.0.0-linux-avx2",
                53_608_928,
                "bd3e811ee696cb602d716c48601b3298dd1944624d71ed6f1e1db86c0f0980ee",
            ),
            baseline: binary(
                "https://github.com/Yoshie2000/PlentyChess/releases/download/b-v8.0.0/PlentyChess-8.0.0-linux-generic",
                53_545_768,
                "60e552dfc6274f5bbf1ada171f721ec5e7c08019a031844c542d4e12c91f6f64",
            ),
        },
    },
    CatalogEngine {
        id: "viridithas",
        name: "Viridithas",
        version: "20",
        licence: "AGPL-3.0",
        homepage: "https://github.com/cosmobobak/viridithas",
        windows: Builds {
            avx2: binary(
                "https://github.com/cosmobobak/viridithas/releases/download/v20.0.0/viridithas-20-win-x86-64-v3.exe",
                56_549_376,
                "c7fa86b2b46fe1c34b0ea3715ad3b4a61d3d984ec63d50b1d6b4f187332fbc51",
            ),
            baseline: None,
        },
        linux: Builds {
            avx2: binary(
                "https://github.com/cosmobobak/viridithas/releases/download/v20.0.0/viridithas-20-linux-x86-64-v3",
                57_800_608,
                "eb53de2fde546f3ba294407e3e8986dc3859bb1818f2ef4cf2fd85d077779973",
            ),
            baseline: None,
        },
    },
    CatalogEngine {
        id: "caissa",
        name: "Caissa",
        version: "2.0",
        licence: "MIT",
        homepage: "https://github.com/Witek902/Caissa",
        windows: Builds {
            avx2: binary(
                "https://github.com/Witek902/Caissa/releases/download/2.0/caissa-2.0-x64-avx2.exe",
                79_174_005,
                "043c0925df8c608d0d87b9e6b1c761240ddd1901ee8cba49e346686b28816b97",
            ),
            baseline: binary(
                "https://github.com/Witek902/Caissa/releases/download/2.0/caissa-2.0-x64-sse2.exe",
                79_561_258,
                "be8dd25fd4a1f00040921b54f3564b35101e63409ef7f762e3cdadd6c6386c53",
            ),
        },
        linux: Builds::NONE,
    },
    CatalogEngine {
        id: "alexandria",
        name: "Alexandria",
        version: "9.1.0",
        licence: "GPL-3.0",
        homepage: "https://github.com/PGG106/Alexandria",
        windows: Builds {
            avx2: binary(
                "https://github.com/PGG106/Alexandria/releases/download/v9.1.0/Alexandria-9.1.0-avx2.exe",
                41_060_319,
                "21fe201c21d7178f520fce94c7f9a6e378597b13f5d3a7e8aa00b4c1adf533b6",
            ),
            baseline: None,
        },
        linux: Builds::NONE,
    },
    CatalogEngine {
        id: "berserk",
        name: "Berserk",
        version: "14",
        licence: "GPL-3.0",
        homepage: "https://github.com/jhonnold/berserk",
        windows: Builds {
            avx2: binary(
                "https://github.com/jhonnold/berserk/releases/download/14/berserk-14-avx2.exe",
                26_567_470,
                "129df854a536d72143d431d1b764ddbd7e50e2f54c28e6ac9ffb50d7a85d6462",
            ),
            baseline: binary(
                "https://github.com/jhonnold/berserk/releases/download/14/berserk-14-x86-64.exe",
                26_537_708,
                "84572e3f3c8fb345aebdc63a7658508edce16a6b5d4d144e5bd3d1bfdb909dbe",
            ),
        },
        linux: Builds::NONE,
    },
    CatalogEngine {
        id: "clover",
        name: "Clover",
        version: "9.1",
        licence: "GPL-3.0",
        homepage: "https://github.com/lucametehau/CloverEngine",
        windows: Builds {
            avx2: binary(
                "https://github.com/lucametehau/CloverEngine/releases/download/v9.1/Clover.9.1-avx2.exe",
                19_063_392,
                "b82602dc70a0105cd4110920f69ec7c7c1fd67c7d99cf19174b4dbbd0ac9e394",
            ),
            baseline: binary(
                "https://github.com/lucametehau/CloverEngine/releases/download/v9.1/Clover.9.1-old.exe",
                19_047_928,
                "063718dfd8e6f78afc5d5562f69e31e1cbe2aa7a55c77e6390d1539b4e2fc5ec",
            ),
        },
        linux: Builds::NONE,
    },
];

/// The catalogue's engine with this id.
pub fn catalog_engine(id: &str) -> Option<&'static CatalogEngine> {
    CATALOG.iter().find(|engine| engine.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA256_HEX_LENGTH: usize = 64;

    fn engine(id: &str) -> &'static CatalogEngine {
        catalog_engine(id).unwrap()
    }

    #[test]
    fn the_fastest_build_the_cpu_can_run_is_chosen() {
        let reckless = engine("reckless");
        let fast = reckless.package(Platform::Linux, true).unwrap();
        let plain = reckless.package(Platform::Linux, false).unwrap();

        assert!(fast.url.ends_with("-avx2"));
        assert!(plain.url.ends_with("-generic"));
    }

    #[test]
    fn a_universal_build_runs_on_every_cpu() {
        let stockfish = engine("stockfish");

        assert!(stockfish.package(Platform::Windows, true).is_some());
        assert!(stockfish.package(Platform::Linux, false).is_some());
    }

    #[test]
    fn an_engine_without_a_build_is_not_offered() {
        assert!(engine("viridithas")
            .package(Platform::Windows, false)
            .is_none());
        assert!(engine("caissa").package(Platform::Linux, true).is_none());
    }

    #[test]
    fn every_package_is_pinned() {
        let packages = CATALOG.iter().flat_map(|engine| {
            [
                &engine.windows.avx2,
                &engine.windows.baseline,
                &engine.linux.avx2,
                &engine.linux.baseline,
            ]
        });

        for package in packages.flatten() {
            assert_eq!(package.sha256.len(), SHA256_HEX_LENGTH, "{}", package.url);
            assert!(package.url.starts_with("https://github.com/"));
            assert!(package.bytes > 0);
        }
    }
}
