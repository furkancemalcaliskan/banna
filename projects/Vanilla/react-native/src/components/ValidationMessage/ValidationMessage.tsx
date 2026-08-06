import i18n from '../../utils/i18n';
import React, { forwardRef } from 'react';
import { FormControlErrorText } from '@gluestack-ui/themed';

type ValidationMessageProps = {
  children?: string;
  [key: string]: any;
};

const ValidationMessage: React.FC<ValidationMessageProps> = ({ children, ...props }) =>
  children ? (
    <FormControlErrorText {...props}>
      {i18n.t(children)}
    </FormControlErrorText>
  ) : null;

const Forwarded = forwardRef<any, ValidationMessageProps>((props, ref) => (
  <ValidationMessage {...props} forwardedRef={ref} />
));

export default Forwarded;
