import { BreadcrumbItem } from '@/types/navigation';

interface BreadcrumbConfig {
  [key: string]: {
    label: string;
    translationKey?: string;
  };
}

const breadcrumbConfig: BreadcrumbConfig = {
  dashboard: { label: 'Dashboard', translationKey: 'navigation.dashboard' },
  inventory: { label: 'Inventory', translationKey: 'navigation.inventory' },
  recipes: { label: 'Recipes', translationKey: 'navigation.recipes' },
  'meal-planning': {
    label: 'Meal Planning',
    translationKey: 'navigation.mealPlanning',
  },
  shopping: { label: 'Shopping Lists', translationKey: 'navigation.shopping' },
  settings: { label: 'Settings', translationKey: 'navigation.settings' },
  profile: { label: 'Profile', translationKey: 'settings.profile' },
  search: { label: 'Search', translationKey: 'common.search' },
  favorites: { label: 'Favorites', translationKey: 'recipes.favorites' },
  create: { label: 'Create', translationKey: 'common.add' },
  edit: { label: 'Edit', translationKey: 'common.edit' },
};

export function createBreadcrumbs(
  pathname: string,
  locale: string,
  t: (key: string) => string,
  customItems?: BreadcrumbItem[]
): BreadcrumbItem[] {
  if (customItems) {
    return customItems;
  }

  // Remove locale from pathname
  const pathWithoutLocale = pathname.replace(`/${locale}`, '') || '/';

  // Split path into segments
  const segments = pathWithoutLocale.split('/').filter(Boolean);

  if (!segments.length) {
    return [];
  }

  const breadcrumbs: BreadcrumbItem[] = [];
  let currentPath = `/${locale}`;

  segments.forEach((segment, index) => {
    currentPath += `/${segment}`;
    const isLast = index === segments.length - 1;

    // Get label from config or format segment
    let label = segment;
    const config = breadcrumbConfig[segment];

    if (config) {
      try {
        if (config.translationKey) {
          const translated = t(config.translationKey);
          label = translated;
        } else {
          label = config.label;
        }
      } catch {
        label = config.label;
      }
    } else {
      // Format segment as title case
      label = formatSegmentLabel(segment);
    }

    breadcrumbs.push({
      label,
      href: currentPath,
      isCurrentPage: isLast,
    });
  });

  return breadcrumbs;
}

function formatSegmentLabel(segment: string): string {
  return segment
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function addBreadcrumbConfig(
  key: string,
  config: { label: string; translationKey?: string }
) {
  breadcrumbConfig[key] = config;
}
