const yourIP = 'localhost';
const port = 44334;
const apiUrl = `http://${yourIP}:${port}`;
type EnvConfig = {
  apiUrl: string;
  oAuthConfig: {
    issuer: string;
    clientId: string;
    scope: string;
  };
  localization: {
    defaultResourceName: string;
  };
};

const ENV: { dev: EnvConfig; prod: EnvConfig } = {
  dev: {
    apiUrl: apiUrl,
    oAuthConfig: {
      issuer: apiUrl,
      clientId: 'MyProjectName_Mobile',
      scope: 'offline_access MyProjectName',
    },
    localization: {
      defaultResourceName: 'MyProjectName',
    },
  },
  prod: {
    apiUrl: 'http://localhost:44305',
    oAuthConfig: {
      issuer: 'http://localhost:44305',
      clientId: 'MyProjectName_Mobile',
      scope: 'offline_access MyProjectName',
    },
    localization: {
      defaultResourceName: 'MyProjectName',
    },
  },
};

export const getEnvVars = (): EnvConfig => {
  // eslint-disable-next-line no-undef
  return __DEV__ ? ENV.dev : ENV.prod;
};
