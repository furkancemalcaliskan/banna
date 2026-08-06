import api from './API';
import { getEnvVars } from '../../Environment';

const { oAuthConfig } = getEnvVars();

const getLoginData = (username: string, password: string) => {
  const formData = {
    grant_type: 'password',
    scope: oAuthConfig.scope,
    username: username,
    password: password,
    client_id: oAuthConfig.clientId,
  };

  if (oAuthConfig.clientSecret)
    formData['client_secret'] = oAuthConfig.clientSecret;

  return Object.entries(formData)
    .map(([key, value]) => `${key}=${encodeURIComponent(value)}`)
    .join('&');
};

const getRefreshData = (refreshToken: string) => {
  const formData = {
    grant_type: 'refresh_token',
    scope: oAuthConfig.scope,
    refresh_token: refreshToken,
    client_id: oAuthConfig.clientId,
  };

  if (oAuthConfig.clientSecret)
    formData['client_secret'] = oAuthConfig.clientSecret;

  return Object.entries(formData)
    .map(([key, value]) => `${key}=${encodeURIComponent(value)}`)
    .join('&');
};

export const login = ({ username, password }: { username: string; password: string }) =>
  api({
    method: 'POST',
    url: '/connect/token',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    data: getLoginData(username, password),
    baseURL: oAuthConfig.issuer,
  }).then(({ data }) => data);

export const refresh = (refreshToken: string) =>
  api({
    method: 'POST',
    url: '/connect/token',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    data: getRefreshData(refreshToken),
    baseURL: oAuthConfig.issuer,
  }).then(({ data }) => data);

export const Logout = (
  input: { client_id: string; token: string; token_type_hint?: string } = { client_id: '', token: '', token_type_hint: '' },
) => {
  if (!input.token_type_hint) {
    input.token_type_hint = 'access_token';
  }

  const _data = Object.entries(input)
    .map(([key, value]) => `${key}=${encodeURIComponent(value)}`)
    .join('&');

  return api({
    method: 'POST',
    url: '/connect/revocat',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    data: _data,
    baseURL: oAuthConfig.issuer,
  }).then(({ data }) => data);
};

export const getTenant = (tenantName: string) =>
  api({
    method: 'GET',
    url: `/api/abp/multi-tenancy/tenants/by-name/${tenantName}`,
  }).then(({ data }) => data);

export const getTenantById = (tenantId: string) =>
  api({
    method: 'GET',
    url: `/api/abp/multi-tenancy/tenants/by-id/${tenantId}`,
  }).then(({ data }) => data);
