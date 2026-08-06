import React from 'react';
import {
  Popover,
  PopoverBackdrop,
  PopoverBody,
  PopoverContent,
  Pressable,
  Text,
  VStack,
  Icon,
  HStack,
} from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';
import { usePopoverColors } from '../../theme';

type ContextMenuAction = {
  key?: string;
  label: string;
  iconName?: React.ComponentProps<typeof Ionicons>['name'];
  isDestructive?: boolean;
  onPress?: () => void;
};

type ContextMenuPopoverProps = {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  actions?: ContextMenuAction[];
  placement?: string;
  triggerProps?: Record<string, any>;
  iconColor?: string;
};

const ContextMenuPopover: React.FC<ContextMenuPopoverProps> = ({
  isOpen,
  onOpen,
  onClose,
  actions = [],
  placement = 'bottom right',
  triggerProps = {},
  iconColor,
}) => {
  if (!actions.length) {
    return null;
  }

  const popoverColors = usePopoverColors();
  const { onPress: customOnPress, ...restTriggerProps } = triggerProps;
  const resolvedIconColor = iconColor || popoverColors.iconColor;

  return (
    <Popover
      isOpen={isOpen}
      onOpen={onOpen}
      onClose={onClose}
      placement={placement}
      trigger={(popoverTriggerProps) => (
        <Pressable
          {...restTriggerProps}
          {...popoverTriggerProps}
          onPress={(event) => {
            popoverTriggerProps.onPress?.(event);
            customOnPress?.(event);
          }}
        >
          <Icon as={Ionicons} name="ellipsis-vertical" size="md" color={resolvedIconColor} />
        </Pressable>
      )}
    >
      <PopoverBackdrop onPress={onClose} />
      <PopoverContent
        bg={popoverColors.bg}
        borderColor={popoverColors.borderColor}
        borderWidth={1}
        borderRadius="$lg"
        minWidth={160}
      >
        <PopoverBody>
          <VStack space="xs">
            {actions.map((action) => (
              <Pressable
                key={action.key || action.label}
                px="$3"
                py="$2"
                borderRadius="$md"
                onPress={() => {
                  onClose?.();
                  action.onPress?.();
                }}
                _pressed={{ bg: popoverColors.pressedBg }}
              >
                <HStack space="sm" alignItems="center">
                  {action.iconName ? (
                    <Icon
                      as={Ionicons}
                      name={action.iconName}
                      size="sm"
                      color={
                        action.isDestructive ? '$danger500' : popoverColors.iconColor
                      }
                    />
                  ) : null}
                  <Text
                    color={
                      action.isDestructive ? '$danger500' : popoverColors.textColor
                    }
                    fontWeight="$medium"
                  >
                    {action.label}
                  </Text>
                </HStack>
              </Pressable>
            ))}
          </VStack>
        </PopoverBody>
      </PopoverContent>
    </Popover>
  );
};

export default ContextMenuPopover;
