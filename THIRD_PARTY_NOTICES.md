# Third-party notices

## ABP Framework React Native startup template

Parts of Banna's embedded React Native scaffold are based on and modified from
the open-source ABP Framework 8.3 React Native application template:

- Upstream repository: <https://github.com/abpframework/abp>
- Upstream source: <https://github.com/abpframework/abp/tree/rel-8.3/templates/app/react-native>
- Upstream version line: `rel-8.3`
- Upstream license: GNU Lesser General Public License v3.0 only

The local derivative covers:

- `projects/Vanilla/react-native/`, except the Banna artwork under `assets/`,
  the local notice, and the license copies.

The upstream template was customized and subsequently refactored by Furkan
Cemal Caliskan. Local changes include TypeScript migration, dependency and Expo
updates, UI/component changes, Banna integration, and entity-generation
support. Git history contains the detailed local changes.

The LGPL-3.0-only text supplied for the upstream work is included at
`LICENSES/LGPL-3.0-only.txt`. Because LGPLv3 incorporates GPLv3 terms, the full
GPLv3 text is included at `LICENSES/GPL-3.0-only.txt`. Both are also embedded
into generated React Native projects. The remainder of Banna is licensed under
Apache-2.0 unless a file or directory is identified otherwise.
