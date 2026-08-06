import React, { useMemo, useState } from 'react';
import { Modal, StyleSheet, useWindowDimensions } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import config, { useAppColorMode, useInputColors } from '../../theme';
import {
  Box,
  HStack,
  Icon,
  Pressable,
  ScrollView,
  Text,
} from '@gluestack-ui/themed';
import { Ionicons } from '@expo/vector-icons';

type OptionValue = string | number | boolean | null | undefined;

type Option = {
  value: OptionValue;
  label: string;
};

type ComboBoxProps = {
  value?: OptionValue;
  options?: Option[];
  onValueChange?: (next: OptionValue) => void;
  placeholder?: string;
  isDisabled?: boolean;
};

const ComboBox: React.FC<ComboBoxProps> = ({
  value,
  options = [],
  onValueChange,
  placeholder = 'Select',
  isDisabled = false,
}) => {
  const [open, setOpen] = useState<boolean>(false);
  const insets = useSafeAreaInsets();
  const { height } = useWindowDimensions();
  const activeMode = useAppColorMode();
  const isDark = activeMode === 'dark';
  const inputColors = useInputColors();

  const selected = useMemo<Option | undefined>(
    () => options.find((option) => option.value === value),
    [options, value],
  );
  const label = selected?.label || placeholder;
  const maxHeight = Math.max(220, height - insets.top - insets.bottom - 120);

  return (
    <>
      <Pressable
        isDisabled={isDisabled}
        onPress={() => setOpen(true)}
        borderWidth={1}
        borderRadius="$lg"
        px="$3"
        py="$3"
        bg={inputColors.bg}
        borderColor={inputColors.borderColor}
        _disabled={{ opacity: 0.5 }}
      >
        <HStack alignItems="center" justifyContent="space-between">
          <Text
            numberOfLines={1}
            color={selected ? inputColors.textColor : inputColors.placeholderColor}
          >
            {label}
          </Text>
          <Icon as={Ionicons} name="chevron-down" size="sm" color={inputColors.iconColor} />
        </HStack>
      </Pressable>

      <Modal transparent visible={open} animationType="fade" onRequestClose={() => setOpen(false)}>
        <Pressable
          style={[
            styles.backdrop,
            {
              backgroundColor:
                isDark ? 'rgba(0, 0, 0, 0.7)' : config.tokens.colors.backdropColor,
            },
          ]}
          onPress={() => setOpen(false)}
        />
        <Box
          mx="$4"
          my="auto"
          bg={inputColors.cardBg}
          borderWidth={1}
          borderRadius="$xl"
          borderColor={inputColors.borderColor}
          style={{ maxHeight }}
        >
          <ScrollView>
            {options.map((option) => {
              const isSelected = option.value === value;
              return (
                <Pressable
                  key={String(option.value)}
                  px="$4"
                  py="$3"
                  borderBottomWidth={1}
                  bg={isSelected ? inputColors.activeHighlight : 'transparent'}
                  borderColor={isDark ? '$themeBorderDark' : '$themeBorderDim'}
                  onPress={() => {
                    onValueChange?.(option.value);
                    setOpen(false);
                  }}
                >
                  <HStack alignItems="center" justifyContent="space-between">
                    <Text 
                      fontWeight={isSelected ? '$bold' : '$normal'}
                      color={inputColors.textColor}
                    >
                      {option.label}
                    </Text>
                    {isSelected ? (
                      <Icon
                        as={Ionicons}
                        name="checkmark"
                        size="sm"
                        color="$primary600"
                        _dark={{ color: '$primary300' }}
                      />
                    ) : null}
                  </HStack>
                </Pressable>
              );
            })}
          </ScrollView>
        </Box>
      </Modal>
    </>
  );
};

const styles = StyleSheet.create({
  backdrop: {
    ...StyleSheet.absoluteFillObject,
    backgroundColor: config.tokens.colors.backdropColor,
  },
});

export default ComboBox;
