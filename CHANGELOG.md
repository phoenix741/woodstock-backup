# [2.1.0-alpha.9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.8...v2.1.0-alpha.9) (2026-08-27)


### Bug Fixes

* **archiving:** 🐛 self-heal materialization stuck on a restrictive destination mode ([e123234](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e12323489cab31379a895ae8fccf7a90d206291a))
* **ci:** 🐛 include server-rs and e2e-tests Cargo.toml in release git assets ([0274973](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0274973dea118101d0face7af89807fa9df9f4c7))
* **config:** 🐛 strip trailing slash before compiling exclude/include globs ([d272565](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d272565ddbeb1ccb1c95952f20bb8efd1cc28c6d))
* **deps:** 🔒️ resync Cargo.lock versions for e2e-tests and woodstock-server-rs ([a0bcd5f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a0bcd5f925198d0eace3651ed9196c57a3858325))
* **front:** 🐛 correct Delete event status and rework event detail UI ([60cade6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/60cade6c8e79d1d34abd704df0abb2857dbcc083))

# [2.1.0-alpha.8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.7...v2.1.0-alpha.8) (2026-08-24)


### Bug Fixes

* **archiving:** 🐛 gate permission restore by source/target OS match ([d5ec566](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d5ec5665cec33c7ba000bf00f4d0cf0275a99c5c))
* **archiving:** 🐛 skip root-only test assertion in CI ([381c442](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/381c44234d1bf90e6291d1b88a575edf94476fa7))

# [2.1.0-alpha.7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.6...v2.1.0-alpha.7) (2026-08-23)


### Features

* **events:** 🎉 refonte page Évènements + statuts Cancelled/Aborted ([9d990c9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9d990c9fc0e540a814ef599a634202c2a825ea08))

# [2.1.0-alpha.6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.5...v2.1.0-alpha.6) (2026-08-22)


### Bug Fixes

* 🐛 gérer SIGTERM dans ws_client_daemon pour un arrêt gracieux ([8b21db1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8b21db12f07122ac83bfe7c1c620b44f5867356c))
* **backup:** 🐛 préserver le statut reprenable si la finalisation échoue après cancel/abort ([399b880](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/399b880f189d5d66aaa99538ea15c1950a6e5c04))


### Performance Improvements

* **fsck:** 🐛 accélérer la vérification des chunks manquants et lui donner sa propre étape ([f7caea8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f7caea8411516e30c4bee0946ac331502b4c2afe))

# [2.1.0-alpha.5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.4...v2.1.0-alpha.5) (2026-08-12)


### Bug Fixes

* add timeout on client ([d3cf833](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d3cf833e3cc51b21115577e0e140534dc09b16cf))

# [2.1.0-alpha.4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.3...v2.1.0-alpha.4) (2026-08-12)


### Bug Fixes

* **archive:** 🐛 échec de fichier en mode dir marque l'hôte failed et logue dans le job ([98b1970](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/98b1970a99406ef61c9e426e94d6e32ad95ebd52))

# [2.1.0-alpha.3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.2...v2.1.0-alpha.3) (2026-08-11)


### Features

* **backup:** ✨ auto-guérit les chunks manquants pendant la sauvegarde ([18fc439](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/18fc439d78f7f16a00987daab8b3258053b18afe))

# [2.1.0-alpha.2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.1.0-alpha.1...v2.1.0-alpha.2) (2026-08-10)


### Bug Fixes

* **ci:** 🐛 corrige le chemin des assets de release (client-zip/server-zip n'existent plus) ([c00e987](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c00e987299ccc29c2a1d3c6f1757c7c1bd04bb36))

# [2.1.0-alpha.1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0...v2.1.0-alpha.1) (2026-08-10)


### Bug Fixes

* 🐛 correct Debian packaging (dependencies, naming, and Debian 13 target) ([7f39534](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7f395349ea33397e2d402f5cfdc56c5b2391ffc9))
* 🐛 restore every file when ws_restore gets no --filter ([675d246](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/675d2460cac24cf244420890416d22ff6f7430d0))
* 📝 fix documentation image ([11baa5d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/11baa5d185e6a6bc4999af704afde12ba9733b04))
* 🔐 assert the right extended key usage on host certificates ([356aa75](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/356aa7580d365beb574a2adc7784d070cc2284d4))
* **archiving:** 🐛 corrige pertes de données, cancel et mutualise les métadonnées de restauration ([da1821f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/da1821f07bd4781b68c304e4ae0e0d58bc758af6)), closes [#107](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/107)
* **ci:** 🐛 corrige le job upload qui cherchait encore des artefacts binaries-* ([59a81d7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/59a81d778f893f7a31815375745c120705634570))
* **ci:** 🐛 downgrade le BOM Trivy en CycloneDX 1.6 avant l'upload vers Dependency-Track ([e82692c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e82692c6686d5e2c46fcdefadf25df031dd09adb)), closes [DependencyTrack/dependency-track#5818](https://gogs.shadoware.org/DependencyTrack/dependency-track/issues/5818)
* **ci:** 🐛 install zstd for tests and fix Windows symlink build ([40ead46](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/40ead469196b9c928cd8bd964bf09b2e825ec42f))
* **ci:** 🐛 télécharge les artefacts dans un sous-dossier dédié pour le job upload ([ef08c31](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ef08c311f652e7fffda6d598af8bbc735b0feedf))
* **client:** 🐛 strip a leading UTF-8 BOM when reading the client config ([144685e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/144685e25f02c15ebde622eca692ee1b5b0cd883))
* **progress:** 🐛 améliorer la gestion des erreurs lors de la récupération de l'état des travaux ([54a77e8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/54a77e818dc6c050f1ba639736691feca8eb9d1f))
* **server:** 🐛 generate JWT keys on startup and derive the agent URL from the request Host header ([322a4d3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/322a4d3bb5c90bcafc3fe5112642e926eadc4ece))
* **windows:** 🐛 resolve the VCRUNTIME/CRT static-linking failure on release builds ([4af4b1c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4af4b1c1160ebe967e024df05f801f89a6a62c3b))


### Features

* **archiving:** ✨ add archive profiles with tar export, dedicated jobs and UI ([7954a38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7954a38d08847292627ab5d09761a38155d6b61b))
* **freebsd:** ✨ add FreeBSD platform support ([2d9fe55](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/2d9fe552e932315da293d38585de9a1be657867e))
* **pool:** ✨ add diagnostics for chunks deleted while still referenced ([4b3b587](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4b3b587134a3ce314bdcde4ec6036ab1a7d571b9))
* **tasks:** ✨ add a Cancel button for backup, restore, archive and fsck jobs ([3bbb3d6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3bbb3d6668ecea04d06be726a37a69ea2061bca6))

# [2.0.0](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v1.0.2...v2.0.0) (2026-04-29)


* refactor!: ♻️ move of the backup logic to the Rust core ([621ab9d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/621ab9d4d5007436c71c960b99d5731ab30ccc92))
* refactor!: migrate the entire backend platform from NestJS to Rust ([652cca1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/652cca119771e819acc8814d83cdc0c46f9e9b67))


### Bug Fixes

* :adhesive_bandage: fix getting the agent from gitea ([5f19a89](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5f19a895abaec244273f0a62c64ffb83b1f346d3))
* :bug: fix missing package.json (missing version information on agent) ([2c44372](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/2c4437229e5ba545fc43847ed71b7c9b0e0bc917))
* :bug: fix progression on refcnt and pool chunk ([37fd08b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/37fd08b7969c3b31f992f30eae124438705aecb2))
* :bug: fix refcnt not working ([49a9004](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/49a9004a10e15ac6e2bd7b30200809fb8a177b14))
* :bug: fix using environment variable on woodstock import ([035bf08](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/035bf08556f0fffd2d2dc2bf5d8e55bdc45a5a21))
* :bug: improve lisibility of number of error of a backup, fix bug where woodstock try to make backup even if the host isn't present ([fe7775d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fe7775ddbf399ffbdb75a721f03cc25efcc6f9f4))
* :loud_sound: add log for resolving dns ([746a4bb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/746a4bbecf7f6e6dd94a1d0d5b89b86ff2fc28bf))
* **#38:** :bug: Fix searching chunks in protobuf file. ([d1ccee0](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d1ccee01da5d2793a16aa5b9dd8fd1eef7a1c40d))
* ♻️ use a sandboxed worker for backup ([e259778](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e259778235777d10de84a8c348949ebfe887d869))
* ✅ fix [#63](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/63): lock file sometimes dropped ([540fde4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/540fde4e6d3f4912abead1b0d1fe43af2fa485b5))
* ✨ fix client update on windows and path ([7fb0f73](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7fb0f7346e3ece5bcc35f9c13467c2941ef1fcca))
* ⬆️ upgrade dependencies ([1dbfb59](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1dbfb5995280ac1bf89b4f77846f255548725484))
* 🐛 auto update only on start ([4cab266](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4cab266bb8339eaebc15825d1586da2fa72a8069))
* 🐛 compact will reorder journal (can take more memory) ([0f016a6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0f016a64bd872a2d529b953596e826ff794e0911))
* 🐛 first fix of rust logger. ([59069e6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/59069e616d857e7d56da387b9ec342e067528ec2))
* 🐛 fix [#62](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/62) some files are not imported ([e3f804e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e3f804ef545ab6de734d2d8e9e52fdc2f1743c7e))
* 🐛 fix browsing ([09c1483](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/09c1483418694247d0d5944e5d29dca08bce8021))
* 🐛 fix bug file not found on windows ([7ee154d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7ee154da5c6bc1df8b318f37b07849eedf68822a))
* 🐛 fix build on windows ([7e84132](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7e841324bfbb0b6fe8be7431414f3fa551b23f37))
* 🐛 fix crash of backuppc importer on synchronisation of filelist ([acc046f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/acc046f12f0d77062d503abcb713bad3fac7a948))
* 🐛 fix defining different path for import ([fb2001b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fb2001b4c59ae93817eb78c8a7f0b3c47240813e))
* 🐛 fix expect on mdns server ([63e61b4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/63e61b4ea5a477ac6b1abb240c45a67e1729c6c0))
* 🐛 fix mdns not working because blocked by windows firewall ([6860d3c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6860d3cc0cac8c67e816fc4c2f3302c46a9fa915))
* 🐛 fix path bug ([815bc88](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/815bc88dd353f79bda3450479e32549461ded5f8))
* 🐛 fix timeout on client side ([bc05bc3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/bc05bc33f42980853edff1904b63f6105db4f1a1))
* 🐛 fix timeout on request is too short to download very big file ([d33551e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d33551e8f52f42f8c5cfc190affef86f22bc02aa))
* 🐛 fix timeout that cause cancel ([fbe46c2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fbe46c224fbef881cf55991f73accd29ffe17cff))
* 🐛 remove redlock that cause backup crash ([3547654](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35476549a1cf7655563b9357147f9556eaf2a0c3))
* 🐛 try to fix connection timeout when server going to bed ([ac4a163](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ac4a16309109f9e874fa7c284e48311f0112201e))
* 🐛 try to fix mdns on a windows 11 laptop (with hyper-v, docker, multiple vpn, ...) ([c7ad571](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c7ad5712fbd16e949d48500677c3ae7591dc0170))
* 🐛 when file is modified, send modify entry instead of add (to permit upload of modified part) ([1412c1b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1412c1bb8b6c681fab15521b42a204504fc6fd30))
* 💚 add environment variables for release automation and version ([60ca1d5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/60ca1d54a2f75f9b108f734a9c91011973a6bf0f))
* 💚 fix how graphql work on front due to codegen update ([af2c647](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/af2c647ae3112886008e1c07d798f0942629938f))
* 🔊 add log information to search bug when computer go to sleep ([317ead9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/317ead9c6b19505da89d61dcc063694dd8af76fe))
* 🔖 [#75](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/75) fix version of rust binary ([e07d14c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e07d14c100117fb3da3a78cf2079bcee52efe8b3))
* 🩹 do not log error for empty file hash in refcnt repair (issue [#91](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/91)) ([26dfdb9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/26dfdb9279179ee00109013a20147bf01782526c))
* 🩹 fix agent version when use backuppc ([c50949e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c50949e726a12f004952d05679b42d01a70855ed))
* 🩹 fix timings rendering on log view ([53dfbc5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/53dfbc55d12b529f51786fac2de348de7d5513c5))
* 🩹 fix unzipping archive ([f50c5a6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f50c5a63bcbefc7ddc60c51c3f3eb41a4d39b166))
* 🚑️ connect to client not working anymore ([fde0130](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fde0130493b2139a62119864789871005d8931d8))
* 🚑️ fix chunk calculation ([fb2a1ce](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fb2a1ceac1dd03a2e50053c95748012b4a5c4d2a))
* 🚑️ fix pm2 in docker not starting ([7bb4341](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7bb43411b22b6fd4a94f8da4cd56ed43c10a9bf3))
* 🚑️ fix rustls that crash ([c2a4fff](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c2a4fffeae90b9f0d7c0ba3a9e9aea8325ab0aeb))
* 🚑️ fix the build of docker image ([6a5869b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6a5869b3eb7590dc97fb85d2e90b1ae6f327b62f))
* 0 is a defined value ([363d5c6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/363d5c63dd666c4cb6f63424651028c46708495a))
* **api:** align ClientType serialization between front and back ([12dc0b3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/12dc0b31c07db4781f321ec9c9126113c373cba8)), closes [#104](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/104)
* **backup:** :bug: fix bug on backup import ([f05aba7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f05aba7edd803143cb07073b4e10cc1b78a8815d))
* **backup:** :bug: fix getting chunk for local import ([9983b0a](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9983b0a6ca28a8e92b67f68ecceedd4722db592b))
* **backup:** :bug: fix import of backup share ([0d3251c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0d3251c5b6a6e57e6d8afba0ae59096b473a1c82))
* **backup:** :bug: fix redis lock timeout ([78ba5f7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/78ba5f7a4f355802a89e6d96343c9dcde13cf8eb))
* **client:** restore mTLS direct resolver with reqwest 0.13 ([ff40c7e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ff40c7e641aa05e652c4fd96f87aaa7af3e235a0))
* **client:** The authentification token has a life time of backup timeout ([34ce212](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/34ce212839aa66e05c48abd70f4b302009374545))
* **client:** try to fix backup client ([b5cef9b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b5cef9b7a0269bd4cf3f1d0472b11a549e7c7ecf))
* correction on drone build ([44eaba7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/44eaba78043057a02cb7dd0c674a92b26e274bfe))
* correction on windows timestamp ([adb1033](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/adb103317a76e38d9864054a82fda6fc7513088a))
* docker version ([915b4b9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/915b4b91ebb420249be21fb1e341c8a9205f4165))
* fix client image not created ([4be9b75](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4be9b75e9bc1079b252c3f116144483709d537ea))
* fix wrong certificate hostname and environment variable ([6a26fd1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6a26fd1ab9bf3a0d3657297297972503d9b405d9))
* **front:** :adhesive_bandage: fix pool usage nb chunk that is not representative ([6cc7d4e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6cc7d4e21aee1d91eaf24da5622b5a5bdf0785d8))
* **job-worker:** preserve job logs across retry attempts ([b00444d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b00444d7a10d19746b0317cbabc99d2fb7726fa1))
* **pool:** preserve failed chunk removals in unused index ([5c765ae](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5c765ae25f20edcf631b46427f708cf44f1aa63e))
* **refcnt:** :bug: fix memory consumption when fsck and node 20.1.0 bug ([31d6837](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/31d68375000d15d27712c6c7b245b661f8ee2489))
* repair denied access on starting service ([fea04a1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fea04a14d7985ca39881b1479196046b2c802136))
* reuse chunk information when the chunk is already present ([70f9394](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/70f939415e1e2d2f8cda103eee2b47a268f129c3))
* **scheduler:** register cron-nightly worker in Monitor to trigger CleanupRefcnt ([70c1ff4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/70c1ff4dfc25527ea7078ea61fa0bf576ca7ec68))
* **shared:** :ambulance: fix access to redis for direct resolution ([91451af](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/91451af33267aadd0f404b0d5591149b921f26c1))
* **shared:** :bug: close backup logger after backup ([1aa9b4e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1aa9b4ea527f6ddc25c353c59510a5deef8550dc))
* **shared:** :green_heart: fix versionning of package.json and cargo.toml ([af24bbf](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/af24bbf614dd92835c6cf66956740d4bd81a7f64))
* update certificate path configuration ([17710e5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/17710e51bb73d01025e1c289346e77686aab9083))
* update the dtrack only for valid branch ([062c238](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/062c238168b794ff7e3903e831eee54321e619b5))
* **workflow:** :bug: don't remove uncomplete backup (will be made with rotation) ([25cb92e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/25cb92eff21e27d9c57e85af1595ff4569aa66fa))


### Features

*  graph update ([be81a7c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/be81a7c9e25fb4053ffabc569abeb5278702e64c))
* :recycle: update protocol to use a unique sync method (instead upload + download) : less memory consumption on client ([da68267](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/da68267548c9d10c81c49225c4b75fd8288d0146))
* :sparkles: [#59](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/59) implemente browsing and downloading file from website directly ([1e36ce1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1e36ce13b2c7b684ee6c39439144f5aa44bcb7a4))
* :sparkles: add a button to download agent from the gitea with config ([cae4877](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/cae4877cef3d71deafbf5879818131fdb5873146))
* :sparkles: add xfer log to api ([b12c161](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b12c161a959d2e76b43eff50cc364de656ce25bf))
* :sparkles: date to next backup is calculated for each host to put in screen information ([02a3b1c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/02a3b1c68bb742675fc41d7ddaa6ba06a0001971))
* :sparkles: fix [#55](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/55), save the backup regulary ([595d342](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/595d342802bd8beeefda035a5348dee64112c5ba))
* :sparkles: fix [#60](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/60), cache host and backups to ensure few wakup of drive ([e1ac862](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e1ac8626984882590747af00b26a3690a77e2d3a))
* :sparkles: get the server version from server ([ca81253](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ca812537ce3f7868688494ba16e6b14a5b23f1bc))
* :sparkles: show log on backup ([9214a76](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9214a769894f3a93fbb6a2a532f005b90286cc24))
* :zap: create ui to debian agent and config ([ebf780c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ebf780c0ebd126916ed82e45fb7a9b7a7dab96d7))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - add special file and directory ([73b2f66](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/73b2f66397410900faa2831e4a8da1b557b6e77d))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - add strict null check ([4a69575](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4a69575e41682682bc94bf85cbb9cbdeb81a9f3b))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files ([1b6d13b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1b6d13b1f9ae511b621d382ac152545ec1b67c2e))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files ([082148b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/082148bdfade2fcb60849e6fb91add6c030d46c1))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files - unit test ([5abd821](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5abd8219faa1acd655c9c8e090849d3b63aaafee))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - list files ([4941f61](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4941f6175d0fe0068fe49d9e0ec73a0c5d1bad74))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - list share for a host/backup number ([e71bd51](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e71bd5150ef0d258cc01fc2870524c94940d4163))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - view all file and directory ([3f46c6b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3f46c6b98a6dd5e29b38b12cb22f0b98481b77f5))
* **#19:** removing backup ([9a9fc8c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9a9fc8cb9cd59522478a9b1d98a447b62a1c93a1)), closes [#19](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/19)
* **#38:** bigint int swagger ([0f26f07](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0f26f079dc85553da2d022b4621906e9d57650aa)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Cleaning the pool should be made in a dedicated refcnt ([3582730](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35827300e62750bbd6a3b04943ccceced3e41964)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Cleaning the refcnt ([35c7280](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35c7280df425968571034486817727413c33f064)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** fix the browser of file ([8e76b09](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e76b0909fa02e8abf97c7790cbe1c4425c39383))
* **#38:** major update ([91f3e8c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/91f3e8c9ac751703b85dd7acd6f137002081177d)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** minor update ([68a1c38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/68a1c38aec4ccc2437d0ba2dc5b35850be73e47c)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** patch update ([c3081fe](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c3081fea6fa899bce0a5bd9aa29dfe200bb66abb)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Refactor the process in job service ([c1a0c12](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c1a0c1243f4190dc33a74202e8b7db592013a68f)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** refcnt of the pool in the queue ([b2e2646](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b2e2646266f3c47b1db1ef6288d209e9c96a6e7a)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** update to bull mq ([aaa883d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/aaa883da4cce771fca1cbcba61f738b22dc4392b)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* ⚡️ add the rust lib ([f4dff0d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f4dff0d8cbf0db723ee8be01ad99f9fd871837bb))
* ✨ [#58](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/58) mount directory from pool ([2016696](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/201669613bd20f60e4631e6f42b7101039d4cb89))
* ✨ [#70](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/70): Add read and write events ([b605391](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b605391e7bb052979779fbd3ddc5df84069970c0))
* ✨ add a debian package and a systemd unit ([8d024a4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8d024a4ad42bec6e59b7ed56f637d2e1620a6adf))
* ✨ add a state in the journal entry to trace error ([51d9671](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/51d9671f03d9f225b89d712697a52d919775e780))
* ✨ add icon to client for windows ([724e5d1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/724e5d1ee44a75bb6041a0281bd9c650a74feb05))
* ✨ add restauration ([07022ec](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/07022ec5c8e9ffd23b038b7a09906eeaf1b3bafc))
* ✨ add the snapshot capability to the client (only btrfs) ([4747bc7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4747bc7bed7e961e0123a1ab50197741f3d94d09))
* ✨ add windows service ([721963f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/721963f40e45fe405e2bca6c0c307739c034f4ed))
* ✨ client can have is config set by environment variable ([6fdbe8e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6fdbe8ed794027eb2ebf559a29c60843c4ffbb55))
* ✨ client can have is config set by environment variable ([19334c6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/19334c6a39527aacf7c71055ee2fa1c9aed1908d))
* ✨ excludes host from migration ([36b3e29](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/36b3e298616bbb65dca4ff0a5651918f5c311f2e))
* ✨ fix [#72](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/72) : the goal is show to the user if the host is online or offline ([44e121e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/44e121e79817af01795794d757ddccd08dd8f703))
* ✨ implement self-update with the self-update crates ([0970331](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/09703315e55146553c2cebe48ebe92e528276189))
* ✨ manifest.path sould be serialized to text and not to base64 ([8e60589](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e60589275f2310fc4c45351d14165d341f0d6e7))
* ✨ parsing of metdata add information on success or failure of reading it ([2d686d1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/2d686d17eb8a3eb29ffaf4b43a052ad371b6f86a))
* ✨ parsing on client side will log all error in the journal ([f13bb0e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f13bb0ede721986d0344bf7ec9bddfcd610631ad))
* ✨ process error in the journal/log ([1d70f6e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1d70f6efef3c8d3cd3b30c63f946f9d8269f0787))
* ✨ remplace zlib by zstd as default compression format ([a5a33ea](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a5a33eaee6369f73e190456017253a47d3e77588))
* ✨ replace ping using ping tools by a ping using the client directly ([b995a69](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b995a6965073e9c131d0137ecc2db6d4ff9af223))
* ✨ replace ping using ping tools by a ping using the client directly ([3c59ed9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3c59ed9ff9a1a7db0c2c2030a935ec9d6b6499b4))
* ✨ the expiration will be updated each time the service is called ([87263d8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/87263d88b17f3eab4b3cadc98f8593b9ac467cd4))
* 🎉 fix [#72](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/72) add resolve module to server and client ([1858629](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1858629e53b9b9424040ce54b79248e5b571883d))
* 💄 update the interface of the front of woodstock ([980b3c9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/980b3c9c8814035cc7e5b48126d3eefb06122ae4))
* 💄 update the theme of woodstock ([cd89973](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/cd8997327803206fbf78d3033f0237f3182f2500))
* 🔊 add host and backup number in log informations ([0b1a3d8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0b1a3d8d711e7d4d1f6ea45450fbd5b81b7c553e))
* 🔊 add log on vss snapshot ([d1be4c8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d1be4c8baa337c2f37775c24032952f100dac9c6))
* 🔊 update log for better error handling ([78db36e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/78db36e817b3ae61b6a6765301daa0a55244eab7))
* add a docker image for the client side ([d35b985](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d35b985be223dc521c25f3603bc76cc8df7b285c))
* add a endpoint to clear the cache ([58df2cb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/58df2cbfea62a4bc121f3c32d2ae3f37d417f7ff))
* add a lock system for pool ([769fade](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/769fade19f0a0b5251793798a09d4b994f6d9c8f))
* add benchmarking for testing hashing function ([ca4b2ab](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ca4b2abb7ea22619f7077834ea08f73b5de4a523))
* add check of integrity of the pool ([e7ccbd8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e7ccbd899822f7c2a34d12cf7b67e82312cb66d3))
* add command read file from console for debug purpose ([1d78ca5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1d78ca5c80e4e34971c21860f1cb0caa24ab39df))
* add matomo to view is the application is downloaded ([e3eb572](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e3eb5726c7771448304cb30ae5e086a9f43bbfeb))
* add snapshot method and share records to backup functionality ([74303df](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/74303dfa8c841b0435f7d24153dbd9701c10b0ee))
* add speed chunk progression ([daaeefb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/daaeefb279653904b4fb497b7279d69870a2ee55))
* add timings informations on xfer log ([e87ba63](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e87ba639c829c0a196adfb4f1f7a76dc09e8de76))
* ajout d'une analyse des dependances avec trivy et dtrack ([e05ff70](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e05ff7028f72a3d2eeeb13d4073c4c1d833c870b))
* **auth:** [#24](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/24) add authentification between the client and the server ([6ad50aa](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6ad50aaaf2d4f20235fbb69be2a7c3fbf11a697e))
* backup a directory ([df213a8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/df213a89d52046a326b8b5f17778298773ec068e))
* **backup:** :sparkles: define the max concurent execution for downloading chunk ([c22d45f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c22d45f60dfc8c3b9545596d0eb7ab4fd45c4d65))
* **backup:** fix [#30](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/30) - stabilize the backup process ([7aa0647](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7aa0647846e124cd2d4a9002b5f2bc6d0c196c1c))
* check compressed pool size (+ ESM) ([1ceb902](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1ceb902b5ee072c4d2b2bb1745069280b2046b13))
* **chunk:** Increase the chunk size ([98384ac](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/98384aca4c6a58c460104b0214e0edeecc47b460))
* **client-rs:** add cleanup for orphaned snapshots and expose fs_accessor ([8d0f2bf](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8d0f2bff1ec68e8b9d7d4a7af80ecb455f60ca64))
* **client-rs:** add Windows VSS snapshots for backup lifecycle ([e2784d3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e2784d37fb4d2375b03a540b41f5f5f2ad78a053)), closes [#92](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/92)
* **console:** :sparkles: when importing, date should be propaged to statistics ([4bc92af](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4bc92af97ff68c35bf7ad65ad9fda4f77214d2ff))
* create a new command to change the hash used in a backup directory ([c34e079](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c34e07914b38b69c3e95bf71de9ce14091b73d85))
* direct connection ([5c7437b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5c7437ba5aca03a5c474f7c5972a8549cf0b04d3))
* docker ([d81e12c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d81e12ce4df66a934779b86243d0b388312dccbe))
* **docker:** update docker part ([fcb46e3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fcb46e310b871a29e74dcab4e59fc2503a4d05f6))
* don't return next date if host should not be backupped ([1f51a2e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1f51a2e9c32f4ee2f0d0be40bbdfec3d52fbaa42))
* fix [#69](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/69) add agentVersion on host and backup ([c4bde6b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c4bde6b433badbb2eca73faf3a9427f154c76f76))
* **front:** localize dates and number formatting ([b0da7e7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b0da7e71e14949ee75faad6982cce3fbea4caaf7))
* **front:** upgrade the front to vue 3 ([e304245](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e304245f9df99d0cf9e67b5fd389a42e514f52b8))
* get configuration from rust ([85b9167](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/85b916780ba9d003a7179a8f03659553f343579b))
* gitea actions ([441a051](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/441a051c6461f02a67f5025d272f0aafb9387fa6))
* implement direct DNS resolution by calling the API on the server ([7d84160](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7d841605e7be72e8ea5eeff4406d44470a4bfd02))
* migrate from vuepress to vitepress ([8080f28](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8080f283422a76bef788b977c613d93e38d83e5b))
* move build on docker hub as promotion ([3244559](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3244559ad5afab7e861e3faaa3c46c7e5f379b26))
* permist usage of different hash for chunk ([9b961a9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9b961a995f774b8fd43a343bbf78ff1cc2a5106a))
* Refactor the program with a new protocol based on grpc ([3ddcd60](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3ddcd60ca26fe50a23b01f6688b4e63f12de0d64))
* **retention:** implement GFS retention policy and manual purge trigger ([4ea9ab1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4ea9ab1f56540b25a9066e3a6bc7a518220ba8d2)), closes [#56](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/56)
* **rust:** ✨ create a rust version ([a9e9855](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a9e9855a444e56f730c1a3d5857bc695352086d6))
* **shared:** :sparkles: in the export of the client, we add the executable from gitea ([a41373b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a41373b66ef3d104fa9d515a93f3f158b7329d6c))
* signed commit ([f8d4c37](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f8d4c370fa2c873d86fc3eaf568e6dca047eca97))
* start creating documentation for client authentification ([28977e1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/28977e1417e5cfc5596180b7353a43824f1cd6bc))
* **stats:** [#20](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/20): Add a prometheus exporter, gather statistics from history file ([0bbbb03](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0bbbb03876caa3231c91f4626cadde22c26f0857))
* **test:** Correction of unit test ([66b1062](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/66b1062bef00111deee14f473781c4257cfc4dec))
* try to optimise generation of the sha3_256 ([e8db387](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e8db38781630f533bb60a3732b0596e1bb9abb11))
* update container ([f1b71bc](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f1b71bca94228db7416640f36961cc71f11de4c0))
* update dev container ([e4163d6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e4163d6ed8359740ee3ef475940e38897d20d64d))
* update docker script ([7230e6d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7230e6d84c95cb1fc77a2fae81271cc65da069a1))
* update docker script for vscode ([a2f84e3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a2f84e39aa6698a7a9a346abf0c495b83fedb75d))
* update the doc with good screenshoot ([a23dc43](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a23dc439a697668550675d753bef04d4cf1e4dd6))
* version with multiple worker ([d93c9fe](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d93c9fe972429c014079724abed857a1efd1d49a))


### Performance Improvements

* ⚡️ increase performance of search-chunk by checking REFCNT before checking manifest ([8e6e1b0](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e6e1b079749a208828a6040a6cd1e36d0ae3d5d))
* **backup:** :alembic: increase memory limit for nodejs ([866d5e7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/866d5e78b40fe055681c78464e30a733ca0707a3))
* **backup:** :zap: improve performance by cumulate multiple file in a fifo ([c968dbb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c968dbb27fe25de5e84f39268c1c4eb8e5cc7695))
* change zlib dependence to ensure better performance ([02a3fc2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/02a3fc2d7abb8b9242abfb4737fa3877bad2d123))


### BREAKING CHANGES

* the application no longer ships or relies on the Node.js/NestJS backend. All backend execution paths, API behavior and operational tooling now depend on the Rust implementation.
* The extensive nature of these refactorings introduces significant
breaking changes that will impact existing deployed versions.

- **Core Backup Logic:** The backup logic has been substantially
  refactored and largely moved into the Rust core. This will likely
  affect how client applications and other services interact with the
  backup process.
- **API and GraphQL:** Major refactoring of NestJS components and
  GraphQL queries/fragments means that API contracts have changed.
  Client applications (including the Vue.js frontend) and any external
  integrations will need to be updated to align with the new API
  structure.
- **Data Format (`backup.yml`):** The `completed` field in `backup.yml`
  files has been renamed to `status`. Existing backup metadata files
  will need to be migrated.
- **Data Management (refcnt & Pool):** The introduction and evolution
  of reference counting (`refcnt`) for pool files, along with changes to
  how `fsck` and the cleaner operate, will likely require data migration
  or specific cleanup scripts for existing installations. Data
  structures related to the pool and file metadata have been altered.
  Orphaned `.info` files (without corresponding `.zz` data chunks) may
  exist and require cleanup.
- **Directory Structure:** The project's directory structure has been
  modified, which might affect build scripts or deployment processes.
- **Locking Mechanisms:** Changes to lock management (introduction of
  shared/exclusive locks) might alter how concurrent operations are
  handled and could require adjustments in custom scripts or
  integrations.

Detailed Summary of Changes:

**Features & Enhancements:**
- **Directory Structure Refactoring:** Initial refactoring of the
  project's directory layout.
- **Rust Core Logic:** Significant portions of the backup logic
  previously in JavaScript/TypeScript have been moved to the Rust core
  for improved maintainability.
- **Reference Counting (`refcnt`):**
    - Introduced `refcnt` for files awaiting addition to the pool.
    - Refactored `refcnt` update mechanisms.
    - Improved `refcnt` operations and associated cleanup processes.
    - Added an `ApplyingRefcnt` state to `fsck` and cleaner processes
      to manage this new mechanism.
- **Lock Management:** Implemented support for shared and exclusive
  locks, enhancing concurrency control.
- **Debugging & Logging:** Added more debug information for chunk
  control and improved logging in backup and `fsck` processes.
- **Standardization:**
    - Standardized code comments, log messages, and all user-facing text
      to English.
    - Addressed `clippy` lints for better Rust code quality.
- **Documentation:** Updated documentation, aided by `clippy`
  suggestions.

**Refactorings:**
- **Rust Code:** Extensive refactoring of Rust backup code and overall
  system architecture.
- **JavaScript/NestJS:** Major refactoring of the JavaScript frontend
  components and NestJS backend services.
- **GraphQL:** Refactored GraphQL queries and fragments for backup and
  task management.
- **UI Components:** Simplified BTRFS commands and improved UI
  components like `PoolView` and `TaskCard`.
- **Error Handling:** Unified error handling mechanisms across backup,
  restore, cleaner, and `fsck` tasks.
- **Build & Linting:** Addressed `clippy` lints and fixed client build
  issues.

**Fixes:**

- **Client Build:** Resolved issues preventing the client from building
  correctly.
- **`fsck` Integrity:** Fixed `fsck` integrity problems and improved the
  repair process for reference counts.
- **Pool Operations:** Corrected the reference file path used in pool
  operations.
- **Chunk Control:** Added debug information to aid in diagnosing chunk
  control issues.

**Migration Scripts & Procedures:**

  * **`backup.yml` Migration:** Execute the
    `cli-rs/migrate_completed_field.sh` script to rename the `completed`
    field to `status` in all existing `backup.yml` files. This script
    will update `completed: true` to `status: Completed` and
    `completed: false` to `status: Aborted`.
  * **Orphaned Chunk Info Cleanup:** Run the
    `cli-rs/check_chunk_integrity.sh --delete` script to identify and
    remove any orphaned `.info` files that do not have corresponding
    `.zz` data chunk files in the pool.

# [2.0.0-alpha.59](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.58...v2.0.0-alpha.59) (2026-04-28)


### Bug Fixes

* **api:** align ClientType serialization between front and back ([12dc0b3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/12dc0b31c07db4781f321ec9c9126113c373cba8)), closes [#104](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/104)

# [2.0.0-alpha.58](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.57...v2.0.0-alpha.58) (2026-04-02)


### Bug Fixes

* **scheduler:** register cron-nightly worker in Monitor to trigger CleanupRefcnt ([70c1ff4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/70c1ff4dfc25527ea7078ea61fa0bf576ca7ec68))


### Features

* 🔊 add log on vss snapshot ([d1be4c8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d1be4c8baa337c2f37775c24032952f100dac9c6))
* add snapshot method and share records to backup functionality ([74303df](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/74303dfa8c841b0435f7d24153dbd9701c10b0ee))

# [2.0.0-alpha.57](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.56...v2.0.0-alpha.57) (2026-03-19)


### Features

* **client-rs:** add cleanup for orphaned snapshots and expose fs_accessor ([8d0f2bf](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8d0f2bff1ec68e8b9d7d4a7af80ecb455f60ca64))
* **client-rs:** add Windows VSS snapshots for backup lifecycle ([e2784d3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e2784d37fb4d2375b03a540b41f5f5f2ad78a053)), closes [#92](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/92)

# [2.0.0-alpha.56](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.55...v2.0.0-alpha.56) (2026-03-15)


### Bug Fixes

* **client:** restore mTLS direct resolver with reqwest 0.13 ([ff40c7e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ff40c7e641aa05e652c4fd96f87aaa7af3e235a0))
* **job-worker:** preserve job logs across retry attempts ([b00444d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b00444d7a10d19746b0317cbabc99d2fb7726fa1))

# [2.0.0-alpha.55](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.54...v2.0.0-alpha.55) (2026-03-13)


### Bug Fixes

* **pool:** preserve failed chunk removals in unused index ([5c765ae](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5c765ae25f20edcf631b46427f708cf44f1aa63e))

# [2.0.0-alpha.54](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.53...v2.0.0-alpha.54) (2026-03-09)


### Bug Fixes

* 🚑️ fix the build of docker image ([6a5869b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6a5869b3eb7590dc97fb85d2e90b1ae6f327b62f))

# [2.0.0-alpha.53](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.52...v2.0.0-alpha.53) (2026-03-09)


### Bug Fixes

* ⬆️ upgrade dependencies ([1dbfb59](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1dbfb5995280ac1bf89b4f77846f255548725484))

# [2.0.0-alpha.52](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.51...v2.0.0-alpha.52) (2026-03-09)


* refactor!: migrate the entire backend platform from NestJS to Rust ([652cca1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/652cca119771e819acc8814d83cdc0c46f9e9b67))


### Features

* **front:** localize dates and number formatting ([b0da7e7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b0da7e71e14949ee75faad6982cce3fbea4caaf7))
* **retention:** implement GFS retention policy and manual purge trigger ([4ea9ab1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4ea9ab1f56540b25a9066e3a6bc7a518220ba8d2)), closes [#56](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/56)


### BREAKING CHANGES

* the application no longer ships or relies on the Node.js/NestJS backend. All backend execution paths, API behavior and operational tooling now depend on the Rust implementation.

# [2.0.0-alpha.51](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.50...v2.0.0-alpha.51) (2025-07-19)


### Bug Fixes

* ♻️ use a sandboxed worker for backup ([e259778](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e259778235777d10de84a8c348949ebfe887d869))

# [2.0.0-alpha.50](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.49...v2.0.0-alpha.50) (2025-07-17)


### Bug Fixes

* fix client image not created ([4be9b75](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4be9b75e9bc1079b252c3f116144483709d537ea))

# [2.0.0-alpha.49](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.48...v2.0.0-alpha.49) (2025-07-17)


### Features

* ✨ client can have is config set by environment variable ([6fdbe8e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6fdbe8ed794027eb2ebf559a29c60843c4ffbb55))

# [2.0.0-alpha.48](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.47...v2.0.0-alpha.48) (2025-07-17)


### Features

* ✨ client can have is config set by environment variable ([19334c6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/19334c6a39527aacf7c71055ee2fa1c9aed1908d))

# [2.0.0-alpha.47](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.46...v2.0.0-alpha.47) (2025-07-16)


### Bug Fixes

* update the dtrack only for valid branch ([062c238](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/062c238168b794ff7e3903e831eee54321e619b5))

# [2.0.0-alpha.46](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.45...v2.0.0-alpha.46) (2025-07-11)


### Features

* ajout d'une analyse des dependances avec trivy et dtrack ([e05ff70](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e05ff7028f72a3d2eeeb13d4073c4c1d833c870b))

# [2.0.0-alpha.45](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.44...v2.0.0-alpha.45) (2025-07-05)


### Bug Fixes

* 🚑️ fix rustls that crash ([c2a4fff](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c2a4fffeae90b9f0d7c0ba3a9e9aea8325ab0aeb))

# [2.0.0-alpha.44](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.43...v2.0.0-alpha.44) (2025-07-05)


### Features

* ✨ add the snapshot capability to the client (only btrfs) ([4747bc7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4747bc7bed7e961e0123a1ab50197741f3d94d09))
* ✨ remplace zlib by zstd as default compression format ([a5a33ea](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a5a33eaee6369f73e190456017253a47d3e77588))
* 🔊 update log for better error handling ([78db36e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/78db36e817b3ae61b6a6765301daa0a55244eab7))


### Performance Improvements

* change zlib dependence to ensure better performance ([02a3fc2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/02a3fc2d7abb8b9242abfb4737fa3877bad2d123))

# [2.0.0-alpha.43](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.42...v2.0.0-alpha.43) (2025-06-22)


### Performance Improvements

* ⚡️ increase performance of search-chunk by checking REFCNT before checking manifest ([8e6e1b0](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e6e1b079749a208828a6040a6cd1e36d0ae3d5d))

# [2.0.0-alpha.42](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.41...v2.0.0-alpha.42) (2025-06-21)


### Bug Fixes

* 🐛 first fix of rust logger. ([59069e6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/59069e616d857e7d56da387b9ec342e067528ec2))
* 🐛 remove redlock that cause backup crash ([3547654](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35476549a1cf7655563b9357147f9556eaf2a0c3))
* 🩹 do not log error for empty file hash in refcnt repair (issue [#91](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/91)) ([26dfdb9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/26dfdb9279179ee00109013a20147bf01782526c))

# [2.0.0-alpha.41](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.40...v2.0.0-alpha.41) (2025-06-16)


### Bug Fixes

* 🚑️ fix pm2 in docker not starting ([7bb4341](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7bb43411b22b6fd4a94f8da4cd56ed43c10a9bf3))

# [2.0.0-alpha.40](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.39...v2.0.0-alpha.40) (2025-06-16)


* refactor!: ♻️ move of the backup logic to the Rust core ([621ab9d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/621ab9d4d5007436c71c960b99d5731ab30ccc92))


### BREAKING CHANGES

* The extensive nature of these refactorings introduces significant
breaking changes that will impact existing deployed versions.

- **Core Backup Logic:** The backup logic has been substantially
  refactored and largely moved into the Rust core. This will likely
  affect how client applications and other services interact with the
  backup process.
- **API and GraphQL:** Major refactoring of NestJS components and
  GraphQL queries/fragments means that API contracts have changed.
  Client applications (including the Vue.js frontend) and any external
  integrations will need to be updated to align with the new API
  structure.
- **Data Format (`backup.yml`):** The `completed` field in `backup.yml`
  files has been renamed to `status`. Existing backup metadata files
  will need to be migrated.
- **Data Management (refcnt & Pool):** The introduction and evolution
  of reference counting (`refcnt`) for pool files, along with changes to
  how `fsck` and the cleaner operate, will likely require data migration
  or specific cleanup scripts for existing installations. Data
  structures related to the pool and file metadata have been altered.
  Orphaned `.info` files (without corresponding `.zz` data chunks) may
  exist and require cleanup.
- **Directory Structure:** The project's directory structure has been
  modified, which might affect build scripts or deployment processes.
- **Locking Mechanisms:** Changes to lock management (introduction of
  shared/exclusive locks) might alter how concurrent operations are
  handled and could require adjustments in custom scripts or
  integrations.

Detailed Summary of Changes:

**Features & Enhancements:**
- **Directory Structure Refactoring:** Initial refactoring of the
  project's directory layout.
- **Rust Core Logic:** Significant portions of the backup logic
  previously in JavaScript/TypeScript have been moved to the Rust core
  for improved maintainability.
- **Reference Counting (`refcnt`):**
    - Introduced `refcnt` for files awaiting addition to the pool.
    - Refactored `refcnt` update mechanisms.
    - Improved `refcnt` operations and associated cleanup processes.
    - Added an `ApplyingRefcnt` state to `fsck` and cleaner processes
      to manage this new mechanism.
- **Lock Management:** Implemented support for shared and exclusive
  locks, enhancing concurrency control.
- **Debugging & Logging:** Added more debug information for chunk
  control and improved logging in backup and `fsck` processes.
- **Standardization:**
    - Standardized code comments, log messages, and all user-facing text
      to English.
    - Addressed `clippy` lints for better Rust code quality.
- **Documentation:** Updated documentation, aided by `clippy`
  suggestions.

**Refactorings:**
- **Rust Code:** Extensive refactoring of Rust backup code and overall
  system architecture.
- **JavaScript/NestJS:** Major refactoring of the JavaScript frontend
  components and NestJS backend services.
- **GraphQL:** Refactored GraphQL queries and fragments for backup and
  task management.
- **UI Components:** Simplified BTRFS commands and improved UI
  components like `PoolView` and `TaskCard`.
- **Error Handling:** Unified error handling mechanisms across backup,
  restore, cleaner, and `fsck` tasks.
- **Build & Linting:** Addressed `clippy` lints and fixed client build
  issues.

**Fixes:**

- **Client Build:** Resolved issues preventing the client from building
  correctly.
- **`fsck` Integrity:** Fixed `fsck` integrity problems and improved the
  repair process for reference counts.
- **Pool Operations:** Corrected the reference file path used in pool
  operations.
- **Chunk Control:** Added debug information to aid in diagnosing chunk
  control issues.

**Migration Scripts & Procedures:**

  * **`backup.yml` Migration:** Execute the
    `cli-rs/migrate_completed_field.sh` script to rename the `completed`
    field to `status` in all existing `backup.yml` files. This script
    will update `completed: true` to `status: Completed` and
    `completed: false` to `status: Aborted`.
  * **Orphaned Chunk Info Cleanup:** Run the
    `cli-rs/check_chunk_integrity.sh --delete` script to identify and
    remove any orphaned `.info` files that do not have corresponding
    `.zz` data chunk files in the pool.

# [2.0.0-alpha.39](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.38...v2.0.0-alpha.39) (2025-04-21)


### Features

* update the doc with good screenshoot ([a23dc43](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a23dc439a697668550675d753bef04d4cf1e4dd6))

# [2.0.0-alpha.38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.37...v2.0.0-alpha.38) (2025-04-19)


### Bug Fixes

* correction on windows timestamp ([adb1033](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/adb103317a76e38d9864054a82fda6fc7513088a))


### Features

* 💄 update the interface of the front of woodstock ([980b3c9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/980b3c9c8814035cc7e5b48126d3eefb06122ae4))
* 💄 update the theme of woodstock ([cd89973](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/cd8997327803206fbf78d3033f0237f3182f2500))

# [2.0.0-alpha.37](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.36...v2.0.0-alpha.37) (2025-04-17)


### Features

* ✨ add icon to client for windows ([724e5d1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/724e5d1ee44a75bb6041a0281bd9c650a74feb05))

# [2.0.0-alpha.36](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.35...v2.0.0-alpha.36) (2025-04-17)


### Bug Fixes

* 🩹 fix timings rendering on log view ([53dfbc5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/53dfbc55d12b529f51786fac2de348de7d5513c5))

# [2.0.0-alpha.35](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.34...v2.0.0-alpha.35) (2025-04-16)


### Bug Fixes

* 🚑️ fix chunk calculation ([fb2a1ce](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fb2a1ceac1dd03a2e50053c95748012b4a5c4d2a))

# [2.0.0-alpha.34](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.33...v2.0.0-alpha.34) (2025-04-15)


### Features

* add benchmarking for testing hashing function ([ca4b2ab](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ca4b2abb7ea22619f7077834ea08f73b5de4a523))
* add command read file from console for debug purpose ([1d78ca5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1d78ca5c80e4e34971c21860f1cb0caa24ab39df))
* add timings informations on xfer log ([e87ba63](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e87ba639c829c0a196adfb4f1f7a76dc09e8de76))
* create a new command to change the hash used in a backup directory ([c34e079](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c34e07914b38b69c3e95bf71de9ce14091b73d85))
* get configuration from rust ([85b9167](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/85b916780ba9d003a7179a8f03659553f343579b))
* permist usage of different hash for chunk ([9b961a9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9b961a995f774b8fd43a343bbf78ff1cc2a5106a))
* try to optimise generation of the sha3_256 ([e8db387](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e8db38781630f533bb60a3732b0596e1bb9abb11))

# [2.0.0-alpha.33](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.32...v2.0.0-alpha.33) (2025-04-05)


### Bug Fixes

* **shared:** :ambulance: fix access to redis for direct resolution ([91451af](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/91451af33267aadd0f404b0d5591149b921f26c1))

# [2.0.0-alpha.32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.31...v2.0.0-alpha.32) (2025-04-04)


### Bug Fixes

* fix wrong certificate hostname and environment variable ([6a26fd1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6a26fd1ab9bf3a0d3657297297972503d9b405d9))

# [2.0.0-alpha.31](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.30...v2.0.0-alpha.31) (2025-04-04)


### Bug Fixes

* 🐛 try to fix mdns on a windows 11 laptop (with hyper-v, docker, multiple vpn, ...) ([c7ad571](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c7ad5712fbd16e949d48500677c3ae7591dc0170))


### Features

* direct connection ([5c7437b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5c7437ba5aca03a5c474f7c5972a8549cf0b04d3))
* implement direct DNS resolution by calling the API on the server ([7d84160](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7d841605e7be72e8ea5eeff4406d44470a4bfd02))

# [2.0.0-alpha.30](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.29...v2.0.0-alpha.30) (2024-12-06)


### Bug Fixes

* 🐛 fix bug file not found on windows ([7ee154d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7ee154da5c6bc1df8b318f37b07849eedf68822a))
* 🐛 fix expect on mdns server ([63e61b4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/63e61b4ea5a477ac6b1abb240c45a67e1729c6c0))

# [2.0.0-alpha.29](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.28...v2.0.0-alpha.29) (2024-11-28)


### Bug Fixes

* ✨ fix client update on windows and path ([7fb0f73](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7fb0f7346e3ece5bcc35f9c13467c2941ef1fcca))

# [2.0.0-alpha.28](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.27...v2.0.0-alpha.28) (2024-11-24)


### Features

* ✨ excludes host from migration ([36b3e29](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/36b3e298616bbb65dca4ff0a5651918f5c311f2e))

# [2.0.0-alpha.27](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.26...v2.0.0-alpha.27) (2024-11-24)


### Bug Fixes

* reuse chunk information when the chunk is already present ([70f9394](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/70f939415e1e2d2f8cda103eee2b47a268f129c3))


### Features

* add a docker image for the client side ([d35b985](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d35b985be223dc521c25f3603bc76cc8df7b285c))
* add a endpoint to clear the cache ([58df2cb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/58df2cbfea62a4bc121f3c32d2ae3f37d417f7ff))
* migrate from vuepress to vitepress ([8080f28](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8080f283422a76bef788b977c613d93e38d83e5b))
* start creating documentation for client authentification ([28977e1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/28977e1417e5cfc5596180b7353a43824f1cd6bc))
* update dev container ([e4163d6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e4163d6ed8359740ee3ef475940e38897d20d64d))

# [2.0.0-alpha.26](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.25...v2.0.0-alpha.26) (2024-11-16)


### Bug Fixes

* 🐛 fix browsing ([09c1483](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/09c1483418694247d0d5944e5d29dca08bce8021))
* repair denied access on starting service ([fea04a1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fea04a14d7985ca39881b1479196046b2c802136))

# [2.0.0-alpha.25](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.24...v2.0.0-alpha.25) (2024-11-09)


### Bug Fixes

* 🐛 fix path bug ([815bc88](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/815bc88dd353f79bda3450479e32549461ded5f8))

# [2.0.0-alpha.24](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.23...v2.0.0-alpha.24) (2024-11-08)


### Bug Fixes

* 🐛 auto update only on start ([4cab266](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4cab266bb8339eaebc15825d1586da2fa72a8069))
* 🐛 compact will reorder journal (can take more memory) ([0f016a6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0f016a64bd872a2d529b953596e826ff794e0911))
* 🐛 fix mdns not working because blocked by windows firewall ([6860d3c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6860d3cc0cac8c67e816fc4c2f3302c46a9fa915))
* 💚 fix how graphql work on front due to codegen update ([af2c647](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/af2c647ae3112886008e1c07d798f0942629938f))
* 🩹 fix agent version when use backuppc ([c50949e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c50949e726a12f004952d05679b42d01a70855ed))


### Features

* :sparkles: [#59](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/59) implemente browsing and downloading file from website directly ([1e36ce1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1e36ce13b2c7b684ee6c39439144f5aa44bcb7a4))
* ✨ add restauration ([07022ec](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/07022ec5c8e9ffd23b038b7a09906eeaf1b3bafc))

# [2.0.0-alpha.23](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.22...v2.0.0-alpha.23) (2024-10-18)


### Bug Fixes

* 🩹 fix unzipping archive ([f50c5a6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f50c5a63bcbefc7ddc60c51c3f3eb41a4d39b166))

# [2.0.0-alpha.22](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.21...v2.0.0-alpha.22) (2024-10-18)


### Bug Fixes

* :adhesive_bandage: fix getting the agent from gitea ([5f19a89](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5f19a895abaec244273f0a62c64ffb83b1f346d3))

# [2.0.0-alpha.21](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.20...v2.0.0-alpha.21) (2024-10-15)


### Bug Fixes

* 💚 add environment variables for release automation and version ([60ca1d5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/60ca1d54a2f75f9b108f734a9c91011973a6bf0f))
* 🔖 [#75](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/75) fix version of rust binary ([e07d14c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e07d14c100117fb3da3a78cf2079bcee52efe8b3))


### Features

* ✨ implement self-update with the self-update crates ([0970331](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/09703315e55146553c2cebe48ebe92e528276189))

# [2.0.0-alpha.20](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.19...v2.0.0-alpha.20) (2024-10-11)


### Features

* ✨ [#58](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/58) mount directory from pool ([2016696](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/201669613bd20f60e4631e6f42b7101039d4cb89))

# [2.0.0-alpha.19](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.18...v2.0.0-alpha.19) (2024-10-01)


### Bug Fixes

* :bug: fix using environment variable on woodstock import ([035bf08](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/035bf08556f0fffd2d2dc2bf5d8e55bdc45a5a21))

# [2.0.0-alpha.18](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.17...v2.0.0-alpha.18) (2024-09-30)


### Bug Fixes

* :bug: fix progression on refcnt and pool chunk ([37fd08b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/37fd08b7969c3b31f992f30eae124438705aecb2))

# [2.0.0-alpha.17](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.16...v2.0.0-alpha.17) (2024-09-30)


### Bug Fixes

* :bug: fix refcnt not working ([49a9004](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/49a9004a10e15ac6e2bd7b30200809fb8a177b14))
* 🐛 fix defining different path for import ([fb2001b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fb2001b4c59ae93817eb78c8a7f0b3c47240813e))

# [2.0.0-alpha.16](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.15...v2.0.0-alpha.16) (2024-09-29)


### Bug Fixes

* 🚑️ connect to client not working anymore ([fde0130](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fde0130493b2139a62119864789871005d8931d8))

# [2.0.0-alpha.15](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.14...v2.0.0-alpha.15) (2024-09-29)


### Features

* ✨ [#70](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/70): Add read and write events ([b605391](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b605391e7bb052979779fbd3ddc5df84069970c0))

# [2.0.0-alpha.14](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.13...v2.0.0-alpha.14) (2024-09-11)


### Features

* ✨ fix [#72](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/72) : the goal is show to the user if the host is online or offline ([44e121e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/44e121e79817af01795794d757ddccd08dd8f703))

# [2.0.0-alpha.13](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.12...v2.0.0-alpha.13) (2024-09-08)


### Bug Fixes

* 🐛 fix timeout on request is too short to download very big file ([d33551e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d33551e8f52f42f8c5cfc190affef86f22bc02aa))
* **workflow:** :bug: don't remove uncomplete backup (will be made with rotation) ([25cb92e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/25cb92eff21e27d9c57e85af1595ff4569aa66fa))


### Features

* 🎉 fix [#72](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/72) add resolve module to server and client ([1858629](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1858629e53b9b9424040ce54b79248e5b571883d))
* fix [#69](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/69) add agentVersion on host and backup ([c4bde6b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c4bde6b433badbb2eca73faf3a9427f154c76f76))

# [2.0.0-alpha.12](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.11...v2.0.0-alpha.12) (2024-09-06)


### Bug Fixes

* 🐛 fix crash of backuppc importer on synchronisation of filelist ([acc046f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/acc046f12f0d77062d503abcb713bad3fac7a948))

# [2.0.0-alpha.11](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.10...v2.0.0-alpha.11) (2024-09-04)


### Bug Fixes

* update certificate path configuration ([17710e5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/17710e51bb73d01025e1c289346e77686aab9083))

# [2.0.0-alpha.10](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.9...v2.0.0-alpha.10) (2024-09-02)


### Bug Fixes

* :loud_sound: add log for resolving dns ([746a4bb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/746a4bbecf7f6e6dd94a1d0d5b89b86ff2fc28bf))
* 🐛 fix timeout that cause cancel ([fbe46c2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fbe46c224fbef881cf55991f73accd29ffe17cff))

# [2.0.0-alpha.9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.8...v2.0.0-alpha.9) (2024-09-01)


### Bug Fixes

* :bug: improve lisibility of number of error of a backup, fix bug where woodstock try to make backup even if the host isn't present ([fe7775d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fe7775ddbf399ffbdb75a721f03cc25efcc6f9f4))
* 🐛 fix build on windows ([7e84132](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7e841324bfbb0b6fe8be7431414f3fa551b23f37))
* 🐛 fix timeout on client side ([bc05bc3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/bc05bc33f42980853edff1904b63f6105db4f1a1))
* 🐛 when file is modified, send modify entry instead of add (to permit upload of modified part) ([1412c1b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1412c1bb8b6c681fab15521b42a204504fc6fd30))


### Features

* :recycle: update protocol to use a unique sync method (instead upload + download) : less memory consumption on client ([da68267](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/da68267548c9d10c81c49225c4b75fd8288d0146))
* :sparkles: add xfer log to api ([b12c161](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b12c161a959d2e76b43eff50cc364de656ce25bf))
* :sparkles: show log on backup ([9214a76](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9214a769894f3a93fbb6a2a532f005b90286cc24))
* ✨ add a state in the journal entry to trace error ([51d9671](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/51d9671f03d9f225b89d712697a52d919775e780))
* ✨ parsing of metdata add information on success or failure of reading it ([2d686d1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/2d686d17eb8a3eb29ffaf4b43a052ad371b6f86a))
* ✨ parsing on client side will log all error in the journal ([f13bb0e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f13bb0ede721986d0344bf7ec9bddfcd610631ad))
* ✨ process error in the journal/log ([1d70f6e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1d70f6efef3c8d3cd3b30c63f946f9d8269f0787))
* ✨ the expiration will be updated each time the service is called ([87263d8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/87263d88b17f3eab4b3cadc98f8593b9ac467cd4))
* 🔊 add host and backup number in log informations ([0b1a3d8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0b1a3d8d711e7d4d1f6ea45450fbd5b81b7c553e))
* don't return next date if host should not be backupped ([1f51a2e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1f51a2e9c32f4ee2f0d0be40bbdfec3d52fbaa42))

# [2.0.0-alpha.8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.7...v2.0.0-alpha.8) (2024-08-21)


### Bug Fixes

* 🐛 try to fix connection timeout when server going to bed ([ac4a163](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ac4a16309109f9e874fa7c284e48311f0112201e))

# [2.0.0-alpha.7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.6...v2.0.0-alpha.7) (2024-08-16)


### Features

* ✨ replace ping using ping tools by a ping using the client directly ([b995a69](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b995a6965073e9c131d0137ecc2db6d4ff9af223))
* ✨ replace ping using ping tools by a ping using the client directly ([3c59ed9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3c59ed9ff9a1a7db0c2c2030a935ec9d6b6499b4))

# [2.0.0-alpha.6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.5...v2.0.0-alpha.6) (2024-08-16)


### Bug Fixes

* 🔊 add log information to search bug when computer go to sleep ([317ead9](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/317ead9c6b19505da89d61dcc063694dd8af76fe))

# [2.0.0-alpha.5](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.4...v2.0.0-alpha.5) (2024-08-15)


### Features

* :sparkles: date to next backup is calculated for each host to put in screen information ([02a3b1c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/02a3b1c68bb742675fc41d7ddaa6ba06a0001971))
* :sparkles: fix [#55](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/55), save the backup regulary ([595d342](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/595d342802bd8beeefda035a5348dee64112c5ba))
* :sparkles: fix [#60](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/60), cache host and backups to ensure few wakup of drive ([e1ac862](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e1ac8626984882590747af00b26a3690a77e2d3a))

# [2.0.0-alpha.4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.3...v2.0.0-alpha.4) (2024-08-14)


### Bug Fixes

* :bug: fix missing package.json (missing version information on agent) ([2c44372](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/2c4437229e5ba545fc43847ed71b7c9b0e0bc917))
* 🐛 fix [#62](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/62) some files are not imported ([e3f804e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e3f804ef545ab6de734d2d8e9e52fdc2f1743c7e))


### Features

* ✨ manifest.path sould be serialized to text and not to base64 ([8e60589](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e60589275f2310fc4c45351d14165d341f0d6e7))

# [2.0.0-alpha.3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.2...v2.0.0-alpha.3) (2024-08-11)


### Bug Fixes

* ✅ fix [#63](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/63): lock file sometimes dropped ([540fde4](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/540fde4e6d3f4912abead1b0d1fe43af2fa485b5))
* **front:** :adhesive_bandage: fix pool usage nb chunk that is not representative ([6cc7d4e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6cc7d4e21aee1d91eaf24da5622b5a5bdf0785d8))


### Features

* :sparkles: add a button to download agent from the gitea with config ([cae4877](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/cae4877cef3d71deafbf5879818131fdb5873146))
* :sparkles: get the server version from server ([ca81253](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ca812537ce3f7868688494ba16e6b14a5b23f1bc))
* :zap: create ui to debian agent and config ([ebf780c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/ebf780c0ebd126916ed82e45fb7a9b7a7dab96d7))

# [2.0.0-alpha.2](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v2.0.0-alpha.1...v2.0.0-alpha.2) (2024-08-06)


### Bug Fixes

* **shared:** :green_heart: fix versionning of package.json and cargo.toml ([bb5f8d8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/bb5f8d8bef3827d54703f62999479084a881f665))


### Features

* ✨ add a debian package and a systemd unit ([8c6f4ca](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8c6f4ca082cfb4894b87ecf0a87840c4ab3c439a))
* ✨ add windows service ([a360f8a](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a360f8a05f69f5ea3f73acd5a96e0969c14bb5e6))
* **shared:** :sparkles: in the export of the client, we add the executable from gitea ([e74d87c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e74d87cee2e4afdab954768e1e0b9c1a35db278a))

# [2.0.0-alpha.1](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/compare/v1.0.2...v2.0.0-alpha.1) (2024-07-29)


### Bug Fixes

* **#38:** :bug: Fix searching chunks in protobuf file. ([d1ccee0](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d1ccee01da5d2793a16aa5b9dd8fd1eef7a1c40d))
* 0 is a defined value ([363d5c6](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/363d5c63dd666c4cb6f63424651028c46708495a))
* **backup:** :bug: fix bug on backup import ([f05aba7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f05aba7edd803143cb07073b4e10cc1b78a8815d))
* **backup:** :bug: fix getting chunk for local import ([9983b0a](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9983b0a6ca28a8e92b67f68ecceedd4722db592b))
* **backup:** :bug: fix import of backup share ([0d3251c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0d3251c5b6a6e57e6d8afba0ae59096b473a1c82))
* **backup:** :bug: fix redis lock timeout ([78ba5f7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/78ba5f7a4f355802a89e6d96343c9dcde13cf8eb))
* **client:** The authentification token has a life time of backup timeout ([34ce212](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/34ce212839aa66e05c48abd70f4b302009374545))
* **client:** try to fix backup client ([b5cef9b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b5cef9b7a0269bd4cf3f1d0472b11a549e7c7ecf))
* correction on drone build ([44eaba7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/44eaba78043057a02cb7dd0c674a92b26e274bfe))
* docker version ([71f7786](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/71f7786027f1827506deeb4865b72bea77bd3890))
* **refcnt:** :bug: fix memory consumption when fsck and node 20.1.0 bug ([31d6837](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/31d68375000d15d27712c6c7b245b661f8ee2489))
* **shared:** :bug: close backup logger after backup ([1aa9b4e](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1aa9b4ea527f6ddc25c353c59510a5deef8550dc))


### Features

*  graph update ([be81a7c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/be81a7c9e25fb4053ffabc569abeb5278702e64c))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - add special file and directory ([73b2f66](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/73b2f66397410900faa2831e4a8da1b557b6e77d))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - add strict null check ([4a69575](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4a69575e41682682bc94bf85cbb9cbdeb81a9f3b))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files ([1b6d13b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1b6d13b1f9ae511b621d382ac152545ec1b67c2e))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files ([082148b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/082148bdfade2fcb60849e6fb91add6c030d46c1))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - download files - unit test ([5abd821](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/5abd8219faa1acd655c9c8e090849d3b63aaafee))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - list files ([4941f61](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4941f6175d0fe0068fe49d9e0ec73a0c5d1bad74))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - list share for a host/backup number ([e71bd51](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e71bd5150ef0d258cc01fc2870524c94940d4163))
* [#32](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/32) - view all file and directory ([3f46c6b](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3f46c6b98a6dd5e29b38b12cb22f0b98481b77f5))
* **#19:** removing backup ([9a9fc8c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/9a9fc8cb9cd59522478a9b1d98a447b62a1c93a1)), closes [#19](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/19)
* **#38:** bigint int swagger ([0f26f07](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0f26f079dc85553da2d022b4621906e9d57650aa)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Cleaning the pool should be made in a dedicated refcnt ([3582730](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35827300e62750bbd6a3b04943ccceced3e41964)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Cleaning the refcnt ([35c7280](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/35c7280df425968571034486817727413c33f064)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** fix the browser of file ([8e76b09](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/8e76b0909fa02e8abf97c7790cbe1c4425c39383))
* **#38:** major update ([91f3e8c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/91f3e8c9ac751703b85dd7acd6f137002081177d)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** minor update ([68a1c38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/68a1c38aec4ccc2437d0ba2dc5b35850be73e47c)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** patch update ([c3081fe](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c3081fea6fa899bce0a5bd9aa29dfe200bb66abb)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** Refactor the process in job service ([c1a0c12](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c1a0c1243f4190dc33a74202e8b7db592013a68f)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** refcnt of the pool in the queue ([b2e2646](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/b2e2646266f3c47b1db1ef6288d209e9c96a6e7a)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* **#38:** update to bull mq ([aaa883d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/aaa883da4cce771fca1cbcba61f738b22dc4392b)), closes [#38](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/38)
* ⚡️ add the rust lib ([11504cb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/11504cb5833323f743d373c528903f521c0d0496))
* add a lock system for pool ([7541d76](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7541d763a541cdec12886531b77048b3ae2f0ff4))
* add check of integrity of the pool ([e7ccbd8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e7ccbd899822f7c2a34d12cf7b67e82312cb66d3))
* add matomo to view is the application is downloaded ([e3eb572](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e3eb5726c7771448304cb30ae5e086a9f43bbfeb))
* add speed chunk progression ([daaeefb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/daaeefb279653904b4fb497b7279d69870a2ee55))
* **auth:** [#24](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/24) add authentification between the client and the server ([6ad50aa](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/6ad50aaaf2d4f20235fbb69be2a7c3fbf11a697e))
* backup a directory ([df213a8](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/df213a89d52046a326b8b5f17778298773ec068e))
* **backup:** :sparkles: define the max concurent execution for downloading chunk ([c22d45f](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c22d45f60dfc8c3b9545596d0eb7ab4fd45c4d65))
* **backup:** fix [#30](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/30) - stabilize the backup process ([7aa0647](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7aa0647846e124cd2d4a9002b5f2bc6d0c196c1c))
* check compressed pool size (+ ESM) ([1ceb902](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/1ceb902b5ee072c4d2b2bb1745069280b2046b13))
* **chunk:** Increase the chunk size ([98384ac](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/98384aca4c6a58c460104b0214e0edeecc47b460))
* **console:** :sparkles: when importing, date should be propaged to statistics ([4bc92af](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/4bc92af97ff68c35bf7ad65ad9fda4f77214d2ff))
* docker ([d81e12c](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d81e12ce4df66a934779b86243d0b388312dccbe))
* **docker:** update docker part ([fcb46e3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/fcb46e310b871a29e74dcab4e59fc2503a4d05f6))
* **front:** upgrade the front to vue 3 ([e304245](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/e304245f9df99d0cf9e67b5fd389a42e514f52b8))
* gitea actions ([441a051](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/441a051c6461f02a67f5025d272f0aafb9387fa6))
* move build on docker hub as promotion ([3244559](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3244559ad5afab7e861e3faaa3c46c7e5f379b26))
* Refactor the program with a new protocol based on grpc ([3ddcd60](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/3ddcd60ca26fe50a23b01f6688b4e63f12de0d64))
* **rust:** ✨ create a rust version ([78c2f80](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/78c2f8099f75ba94d875290b9ded14facc881e5d))
* signed commit ([f8d4c37](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f8d4c370fa2c873d86fc3eaf568e6dca047eca97))
* **stats:** [#20](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/issues/20): Add a prometheus exporter, gather statistics from history file ([0bbbb03](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/0bbbb03876caa3231c91f4626cadde22c26f0857))
* **test:** Correction of unit test ([66b1062](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/66b1062bef00111deee14f473781c4257cfc4dec))
* update container ([f1b71bc](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/f1b71bca94228db7416640f36961cc71f11de4c0))
* update docker script ([7230e6d](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/7230e6d84c95cb1fc77a2fae81271cc65da069a1))
* update docker script for vscode ([a2f84e3](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/a2f84e39aa6698a7a9a346abf0c495b83fedb75d))
* version with multiple worker ([d93c9fe](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/d93c9fe972429c014079724abed857a1efd1d49a))


### Performance Improvements

* **backup:** :alembic: increase memory limit for nodejs ([866d5e7](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/866d5e78b40fe055681c78464e30a733ca0707a3))
* **backup:** :zap: improve performance by cumulate multiple file in a fifo ([c968dbb](https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/commit/c968dbb27fe25de5e84f39268c1c4eb8e5cc7695))


### BREAKING CHANGES

* **rust:** this is a new version of the client and the server
