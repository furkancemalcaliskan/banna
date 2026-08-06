import {
  Box,
  Checkbox,
  CheckboxIndicator,
  CheckboxIcon,
  CheckboxLabel,
  CheckIcon,
  VStack,
} from '@gluestack-ui/themed';
import React, { useEffect, useState } from 'react';
import { getAllRoles, getUserRoles } from '../../api/IdentityAPI';

type UserRoleItem = {
  id?: string;
  name?: string;
  isDefault?: boolean;
  isSelected?: boolean;
};

type UserRolesProps = {
  editingUser?: Record<string, any>;
  onChangeRoles: (roles: string[]) => void;
};

function UserRoles({ editingUser = {}, onChangeRoles }: UserRolesProps) {
  const [roles, setRoles] = useState<UserRoleItem[]>([]);

  const onPress = index => {
    setRoles(
      roles.map((role, i) => ({
        ...role,
        isSelected: index === i ? !role.isSelected : role.isSelected,
      })),
    );
  };

  useEffect(() => {
    const requests = [getAllRoles()];
    if (editingUser.id) requests.push(getUserRoles(editingUser.id));

    Promise.all(requests).then(([allRoles = [], userRoles = []]) => {
      setRoles(
        allRoles.map(role => ({
          ...role,
          isSelected: editingUser.id
            ? !!userRoles?.find(userRole => userRole?.id === role?.id)
            : role.isDefault,
        })),
      );
    });
  }, [editingUser.id]);

  useEffect(() => {
    onChangeRoles(roles.filter(role => role.isSelected).map(role => role.name));
  }, [roles]);

  return (
    <Box w="$full" px="$4">
      <VStack borderWidth={0}>
        {roles.map((role, index) => (
          <Box key={role.id} borderBottomWidth={1} borderColor="$secondary200" py="$2">
            <Checkbox
              isChecked={role.isSelected}
              onChange={() => onPress(index)}
              value={role.name}
              aria-label={role.name}
            >
              <CheckboxIndicator>
                <CheckboxIcon as={CheckIcon} />
              </CheckboxIndicator>
              <CheckboxLabel> {role.name}</CheckboxLabel>
            </Checkbox>
          </Box>
        ))}
      </VStack>
    </Box>
  );
}

export default UserRoles;
