import { useSelector } from 'react-redux';
import { createGrantedPolicySelector } from '../store/selectors/AppSelectors';

export function usePermission(key?: string | null) {
  const isGranted = useSelector(createGrantedPolicySelector(key));
  return isGranted;
}
