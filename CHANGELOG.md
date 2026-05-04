# Changelog

All notable changes to pgpilot are documented here.
Releases follow [Semantic Versioning](https://semver.org/).

## [0.5.1] — 2026-05-04

### Features

- **all**: bump version to v0.5.1 ([`8a345f4`](https://github.com/gfriloux/pgpilot/commit/8a345f4efe4f66dedcbc81921e6a6a8eeb3d3bc5))
## [0.5.0] — 2026-05-04

### Bug Fixes

- **all**: fix desktop logo ([`27df491`](https://github.com/gfriloux/pgpilot/commit/27df4911e66f940d8fbc0375fc3326e59ec8ea00))
- **ui**: various things related to ussr theme ([`4cb6357`](https://github.com/gfriloux/pgpilot/commit/4cb635717f4701a2bc2bfbcee4641494b5142c2c))

### Documentation

- **mdbook**: first real doc ([`206c08f`](https://github.com/gfriloux/pgpilot/commit/206c08f05a43de174e8d4b4ce7c9cafafc37d8cf))
- **mdbook**: whoops, build output ([`cb2c2e0`](https://github.com/gfriloux/pgpilot/commit/cb2c2e04f1c497013511d77d5ea4fbe203d4dfb0))
- **README**: add desktop files infos ([`76528c5`](https://github.com/gfriloux/pgpilot/commit/76528c51d4e40a60d50a6885f0ca6b5a68d544b9))
- **mdbook**: Add caution about .gnupg ([`b804818`](https://github.com/gfriloux/pgpilot/commit/b804818fabdb4b0f88629b5fff782814fc1c7100))
- **mdbook**: remove french words ([`1240377`](https://github.com/gfriloux/pgpilot/commit/1240377c1fd0c01766e0232d233b1271658dd37b))
- **README**: fix terrible mistake ([`064b8aa`](https://github.com/gfriloux/pgpilot/commit/064b8aaeca02721945127eee4d98a5beabb804af))
- **mdbook**: Add documentation about settings ([`c63e15b`](https://github.com/gfriloux/pgpilot/commit/c63e15b9fbcba4b171d427b0afc86e07be3bf310))
- **README**: Update roadmap, v0.5.0 changes ([`c3239e5`](https://github.com/gfriloux/pgpilot/commit/c3239e5e842e97d06b2ee18e8ad6c10d604e7913))

### Features

- **all**: add i18n support (en/fr) ([`46e0bf6`](https://github.com/gfriloux/pgpilot/commit/46e0bf664d480bc47cd58dd2fee47a6f626c9b5e))
- **all**: add desktop files ([`84aa743`](https://github.com/gfriloux/pgpilot/commit/84aa743e06366aad4727cbeeda9fbea6f68796a5))
- **all**: some UI rework + code review ([`fb0c4a4`](https://github.com/gfriloux/pgpilot/commit/fb0c4a42490bfc8e061ff337bd044ba5ef001d68))
- **ui**: enhance themes ([`74a5b17`](https://github.com/gfriloux/pgpilot/commit/74a5b178d6e796797db054f9cf9d50ff1cb8e986))
- **ui**: add 2 new fonts ([`7318a2b`](https://github.com/gfriloux/pgpilot/commit/7318a2bf580981eb74e77f9ec51193e0e314e966))

### Testing

- **all**: first time having tests ([`6fa6d02`](https://github.com/gfriloux/pgpilot/commit/6fa6d0227d0af781f6392ef51c2ee0dbdbe90c34))
## [0.4.0] — 2026-05-03

### Bug Fixes

- **gpg**: prevent output overwrite and add collision counters ([`918a339`](https://github.com/gfriloux/pgpilot/commit/918a339c996f4672a3109cbfe7c4e4539ff38a2d))
- **errors**: sanitize gpg stderr and truncate long error messages ([`25c10a0`](https://github.com/gfriloux/pgpilot/commit/25c10a05dbd96b91baddf8a6fc0a0473b50159ac))
- **gpg**: rewrite revoke_subkey_at_pos with token-driven dialogue ([`00fa1d4`](https://github.com/gfriloux/pgpilot/commit/00fa1d498aa7e876995bbed2474392d9e6bd6466))
- **gpg**: rollback rotate_subkey keyring if revocation fails ([`9506609`](https://github.com/gfriloux/pgpilot/commit/9506609b803d628c36c21feaa98760e03d3bebf2))
- **gpg**: add SAFETY comment on NullPolicy unsafe block ([`0d194c4`](https://github.com/gfriloux/pgpilot/commit/0d194c44217f059f73742ab5c6ead3744801d077))

### Documentation

- **security**: add threat model (THREAT_MODEL.md) ([`0f33eb1`](https://github.com/gfriloux/pgpilot/commit/0f33eb12e9c5577a0ade383afeda9c0bf10e45ea))
- **readme**: update roadmap ([`4a6fde7`](https://github.com/gfriloux/pgpilot/commit/4a6fde7d3b2c89272d6768abfc619c4169f7f723))

### Features

- **all**: bump version to v0.4.0 ([`6ae7ec9`](https://github.com/gfriloux/pgpilot/commit/6ae7ec972e63341d067bdc014d7ee7de0f3df45d))
- **verify**: show signer trust level in signature result ([`968256d`](https://github.com/gfriloux/pgpilot/commit/968256d55e9fbe1fe4800d3e0f9df182f7769ddb))
- **all**: enforce https-only, redirect cap, and 1MiB body limit ([`4916fa2`](https://github.com/gfriloux/pgpilot/commit/4916fa2272521f9abbc3ccc2b28845269dca4bac))
- **security**: validate fingerprints and keyserver queries at gpg layer ([`74c65cc`](https://github.com/gfriloux/pgpilot/commit/74c65ccad34db85e24ee458c54c7f79bc6c7c32c))

### Refactoring

- **gpg**: isolate env and add --homedir to all gpg calls ([`f2b355b`](https://github.com/gfriloux/pgpilot/commit/f2b355bdb5f116a58733af22222e8a64cdc5eb85))
## [0.3.0] — 2026-05-03

### Bug Fixes

- **verify**: distinguish bad-sig from unknown-key via VerifyOutcome enum ([`043b1cc`](https://github.com/gfriloux/pgpilot/commit/043b1ccf403ddc597daf9a9c70cf0f645cb63b3f))

### Dependencies

- **deps**: Update GitHub Actions ([`9ef34aa`](https://github.com/gfriloux/pgpilot/commit/9ef34aa310ac787abd3355c60e6b85f18050a06b))

### Documentation

- **all**: first version ([`e98327d`](https://github.com/gfriloux/pgpilot/commit/e98327dba411da841789c627d496b54c36e0613a))
- **all**: update ([`26de84d`](https://github.com/gfriloux/pgpilot/commit/26de84d60c7582d1865fa5631c389d3daa904b19))

### Features

- **all**: bump version to v0.3.0 ([`6974f76`](https://github.com/gfriloux/pgpilot/commit/6974f76aadd80325782eeb3afd0a6a3647e4723b))

### Refactoring

- **ui**: extract shared button styles and common widgets ([`9d54c6f`](https://github.com/gfriloux/pgpilot/commit/9d54c6f5a6a5bf174e7797dcb8ca83eff03410cd))
- **ui**: fix semantic color tokens and add theme constants ([`46dea33`](https://github.com/gfriloux/pgpilot/commit/46dea33a4c15085923c065c417794646f1a3e064))
- **ui**: restructure sidebar and add form structure to sign/verify ([`6477ade`](https://github.com/gfriloux/pgpilot/commit/6477ade5f4d37e48b8e55cb88687c076beab1a7d))
- **ui**: improve UX states, drop zone, and modal colors ([`7d9a62f`](https://github.com/gfriloux/pgpilot/commit/7d9a62fbac35ffcfd6584d2832c3ab99fd8072a6))
- **ui**: label État column header in key list ([`e53e7ae`](https://github.com/gfriloux/pgpilot/commit/e53e7ae91d139991f8e4e9c9bba71bc2bc4743cc))
## [0.2.0] — 2026-05-03

### Features

- **all**: bump rfd 0.17 + sequoia-openpgp 2, fix subkey rotation ([`37a252b`](https://github.com/gfriloux/pgpilot/commit/37a252bd975bc73f5e7dd22bc53b03e4dba8db19))
- **all**: migrate ureq 2 → 3 ([`05bf284`](https://github.com/gfriloux/pgpilot/commit/05bf284ce5aa83c7ac6e27032f61a242cb6b5016))
- **ui**: migrate iced 0.13→0.14, rfd 0.17, fix file dialogs on NixOS ([`f62e261`](https://github.com/gfriloux/pgpilot/commit/f62e261f96611ce9b78fc3763b4927895b077497))
## [0.1.0] — 2026-05-03

### Bug Fixes

- **ui**: import menu should be just like others ([`0de6d19`](https://github.com/gfriloux/pgpilot/commit/0de6d19d712e3ce9afcab870066286c1b9388de3))
- **app**: HealthChecksLoaded carries Result, errors no longer swallowed ([`b9e1065`](https://github.com/gfriloux/pgpilot/commit/b9e1065bab92818ee96f7d1d8445cbe5ce2b3dcf))
- **ui**: small details ([`5bb4a3d`](https://github.com/gfriloux/pgpilot/commit/5bb4a3d76884ef72d0c659b01887cd547e3afdc3))

### Features

- **all**: first version ([`2d8cb19`](https://github.com/gfriloux/pgpilot/commit/2d8cb1976489889aa4c07466cc102a702a3a5fc1))
- **all**: updates ([`9ca35c0`](https://github.com/gfriloux/pgpilot/commit/9ca35c0ad76a319575ed0e9f5d4b4c163b242c56))
- **all**: allows to import keys ([`d21ce57`](https://github.com/gfriloux/pgpilot/commit/d21ce572e8e8a55088c5a8142ec80745cc1958ab))
- **ui**: enhance ([`20dd5d8`](https://github.com/gfriloux/pgpilot/commit/20dd5d8d332b36922b5063a99786596993ce37cd))
- **ui**: use nerd fonts ([`8278855`](https://github.com/gfriloux/pgpilot/commit/8278855ad425e3b8c42bdd323e99ff89751bf9b9))
- **all**: add yubikey support ([`f91174f`](https://github.com/gfriloux/pgpilot/commit/f91174f91ecb8839c73ddd615a1875506fa7c812))
- **all**: allow to delete keys ([`43d9deb`](https://github.com/gfriloux/pgpilot/commit/43d9deb21513013191f45bd99dc0b3c0e1b89fac))
- **all**: support subkeys ([`9416228`](https://github.com/gfriloux/pgpilot/commit/94162286549d91143b4152c9f1ec5595eeb55778))
- **all**: allow to renew keys ([`9ae782b`](https://github.com/gfriloux/pgpilot/commit/9ae782b4ac44382ed6b1ccec016e6ed0ad39b3a5))
- **all**: enchance creation of subkeys ([`24c441b`](https://github.com/gfriloux/pgpilot/commit/24c441b0ed36fb7f16dbc37979f67a79ffa6aabf))
- **all**: add support for keyservers ([`4b50396`](https://github.com/gfriloux/pgpilot/commit/4b503965a5c2555d31403fa1caab8f90ceb397f2))
- **all**: republish key when rotating sub keys ([`09213b7`](https://github.com/gfriloux/pgpilot/commit/09213b7e6aab1fd0ce88e64f74f6d412b327d58c))
- **all**: add multiple options for exporting pubkey ([`64d9c83`](https://github.com/gfriloux/pgpilot/commit/64d9c836d51d133d39b79fd5312bb86107b3cec6))
- **all**: merge buttons to update expiry date and replacing subkey ([`1e4f6f8`](https://github.com/gfriloux/pgpilot/commit/1e4f6f8f9d6a3100475c3cd1db4990408fdf541d))
- **all**: enhance importing of keys ([`45558f9`](https://github.com/gfriloux/pgpilot/commit/45558f94e4e1dee3d408d228e942e722f72aa033))
- **all**: add gpg diagnostic page ([`53ad6ec`](https://github.com/gfriloux/pgpilot/commit/53ad6ec4dacd9c41e6797344a0f43625e958a317))
- **ui**: darker mode ([`7ef34c4`](https://github.com/gfriloux/pgpilot/commit/7ef34c4ea8f4bcb5d50ef184122341f293a722c2))
- **ui**: go on catppuccin frappé theme ([`e84b478`](https://github.com/gfriloux/pgpilot/commit/e84b478262af5035cd03352829254ccf03cd51b9))
- **ui**: more catppuccin theme, applied to smaller widgets ([`2d9988a`](https://github.com/gfriloux/pgpilot/commit/2d9988a01ecd842b80f9180e1e5969ceb783ea1e))
- **all**: allow exporting revocation cert with private key ([`78734c7`](https://github.com/gfriloux/pgpilot/commit/78734c731795a3011bc1e41c6bd23b26c8566274))
- **encrypt**: file encryption with trust level management ([`c092b54`](https://github.com/gfriloux/pgpilot/commit/c092b5459b8c274cdd6952c2f78f568f8e14f168))
- **decrypt**: add GPG file decryption view with key inspection ([`7e38c11`](https://github.com/gfriloux/pgpilot/commit/7e38c117b1e553b7fa37bae2d44f8b4f6bf6d2eb))
- **sign**: add file signing and signature verification view ([`6490a02`](https://github.com/gfriloux/pgpilot/commit/6490a025004b180cfda982eeac05b13d640356c7))
- **ui**: split sign/verify into two separate views ([`990bdb4`](https://github.com/gfriloux/pgpilot/commit/990bdb4e5cac401e22b584b493a0b246cf7ac849))
- **ui**: auto-dismiss notifications + dismiss button ([`43023df`](https://github.com/gfriloux/pgpilot/commit/43023dfbde724bb77fb9522f05e6dcea1a2e808c))
- **ui**: master-detail horizontal layout + responsive buttons ([`b670537`](https://github.com/gfriloux/pgpilot/commit/b670537fa65b597f91df510080515a50d2f937cf))

### Refactoring

- **all**: better separation logic ([`8659ced`](https://github.com/gfriloux/pgpilot/commit/8659cedd86a65bbc6744aa445c0d51d2f57c16b0))
- **all**: clean code ([`e8d1d90`](https://github.com/gfriloux/pgpilot/commit/e8d1d906d0f50a9645e5992f008e430008ba757f))
- **all**: clean code ([`d832fac`](https://github.com/gfriloux/pgpilot/commit/d832fac130e05263154f5a4f2afe70f460d600ff))
- **all**: introduce SubkeyType enum, remove raw algo strings ([`e7958cf`](https://github.com/gfriloux/pgpilot/commit/e7958cf0a11de373ee751609bb5526d7a42722cc))
- **app**: split update() into 35 focused handler methods ([`8c35890`](https://github.com/gfriloux/pgpilot/commit/8c358903a82ee8668b3bb926574ffded702e7061))
- **gpg**: centralize gnupg_dir() in mod.rs ([`85e9985`](https://github.com/gfriloux/pgpilot/commit/85e99852241222ecf2f5cb9c35ba09350f2d09e8))
- **app**: use fingerprints instead of indices as key identifiers ([`a1a71ce`](https://github.com/gfriloux/pgpilot/commit/a1a71cece810a8cdc9f75f51e6318907e3977915))
- **ui**: split key_detail::view() into focused private functions ([`b272fd9`](https://github.com/gfriloux/pgpilot/commit/b272fd93b80ffd3a4e9b4818ec79c21a19389e40))
- **app**: typed StatusKind replaces starts_with("Erreur") heuristic ([`7fc6467`](https://github.com/gfriloux/pgpilot/commit/7fc6467a4a5385cd58ef3fc02c2fd08b114b8101))
- **gpg**: use 16-char key_id everywhere, drop insecure 8-char ID ([`eee5f0d`](https://github.com/gfriloux/pgpilot/commit/eee5f0d7807bbf357c54bc6af7bb654d7eb87916))
- **app**: replace 5 pending_* fields with single PendingOp enum ([`c7a4407`](https://github.com/gfriloux/pgpilot/commit/c7a4407449454ae2b69fbee72075139bdc6ae377))
- **app**: split app.rs into domain submodules ([`900e700`](https://github.com/gfriloux/pgpilot/commit/900e700bd477230ae18eb84edaabdfc8ff688a23))

### Style

- **all**: use Shadow::default() explicitly per clippy::pedantic ([`56462b2`](https://github.com/gfriloux/pgpilot/commit/56462b2e26566dbdd0e68d1880f4c520fd5f14d2))

