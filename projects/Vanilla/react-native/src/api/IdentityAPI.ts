import api from './API';

export const getAllRoles = () => api.get('/api/identity/roles/all').then(({ data }) => data.items);

export const getUserRoles = (id: string) =>
  api.get(`/api/identity/users/${id}/roles`).then(({ data }) => data.items);

export const getUsers = (params: { maxResultCount?: number; skipCount?: number } = { maxResultCount: 10, skipCount: 0 }) =>
  api.get('/api/identity/users', { params }).then(({ data }) => data);

export const getUserById = (id: string) => api.get(`/api/identity/users/${id}`).then(({ data }) => data);

export const createUser = (body: any) => api.post('/api/identity/users', body).then(({ data }) => data);

export const updateUser = (body: any, id: string) =>
  api.put(`/api/identity/users/${id}`, body).then(({ data }) => data);

export const removeUser = (id: string) => api.delete(`/api/identity/users/${id}`);

export const getProfileDetail = () => api.get('/api/account/my-profile').then(({ data }) => data);

export const updateProfileDetail = (body: any) =>
  api.put('/api/account/my-profile', body).then(({ data }) => data);

export const changePassword = (body: any) =>
  api.post('/api/account/my-profile/change-password', body).then(({ data }) => data);
