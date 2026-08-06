import React from 'react';
import { Box, Divider } from '@gluestack-ui/themed';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import FormButtons from './FormButtons';

type FormButtonBarProps = React.ComponentProps<typeof FormButtons> & {
  children?: React.ReactNode;
};

function FormButtonBar({ children, ...props }: FormButtonBarProps) {
  const insets = useSafeAreaInsets();
  const paddingBottom = Math.max(insets.bottom || 0, 12);

  return (
    <Box
      position="absolute"
      left="$0"
      right="$0"
      bottom="$0"
      px="$4"
      pt="$3"
      pb={paddingBottom}
      bg="transparent"
    >
      <Box mb="$3">
        <Divider bg="$themeBorderDim" _dark={{ bg: '$themeBorderDark' }} />
      </Box>
      {children || <FormButtons {...props} isFixed={false} />}
    </Box>
  );
}

export default FormButtonBar;
