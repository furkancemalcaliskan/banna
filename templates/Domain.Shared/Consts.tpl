namespace ${namespace}
{
    public static class ${entity_name}Consts
    {
        private const string DefaultSorting = "{0}CreationTime asc";

        public static string GetDefaultSorting(bool withEntityName)
        {
            return string.Format(DefaultSorting, withEntityName ? "${entity_name}." : string.Empty);
        }
        ${consts}
    }
}
