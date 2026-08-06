import React, { forwardRef } from 'react';
import { usePermission } from '../hooks/UsePermission';

type WithPermissionProps = {
  policyKey?: string;
  [key: string]: any;
};

export function withPermission<P extends object>(Component: React.ComponentType<P>, policyKey?: string) {
  const Forwarded = forwardRef<any, P & WithPermissionProps>((props, ref) => {
    const isGranted =
      policyKey || props.policyKey ? usePermission(policyKey || props.policyKey) : true;
    return isGranted ? <Component ref={ref} {...props as P} /> : null;
  });

  return Forwarded;
}
