 using Microsoft.EntityFrameworkCore;
 using Volo.Abp.EntityFrameworkCore.Modeling;
 ${nav_usings}

 namespace ${namespace}
 {
     public static class EfCore${entity_name}ModelCreatingExtension
     {
         public static void Configure${entity_name}(this ModelBuilder builder)
         {
             builder.Entity<${entity_name}>(b =>
             {
                 b.ToTable(${domain_short}Consts.DbTablePrefix + "${entity_plural}", ${domain_short}Consts.DbSchema);
                 b.ConfigureByConvention();
                 ${efcore_property_configs}
                 ${navigation_fk_configs}
             });
         }
     }
 }

