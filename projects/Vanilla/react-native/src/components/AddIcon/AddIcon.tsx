import { Icon, AddIcon as GluestackAddIcon, IconProps } from '@gluestack-ui/themed';
import React from 'react';

type AddIconProps = IconProps & {
  onPress?: () => void;
};

const AddIcon: React.FC<AddIconProps> = ({ onPress, ...iconProps }) => {
  return (
    <Icon
      as={GluestackAddIcon}
      size="xl"
      mr="$3"
      onPress={onPress}
      {...iconProps}
    />
  );
};

export default AddIcon;
