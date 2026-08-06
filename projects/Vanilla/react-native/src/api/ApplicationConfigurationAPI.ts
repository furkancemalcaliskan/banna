import api from './API';

export const getApplicationConfiguration = () =>
  api.get('/api/abp/application-configuration').then(({ data }) => data);
