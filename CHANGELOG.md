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
