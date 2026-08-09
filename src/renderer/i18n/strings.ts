export type Locale = 'en' | 'ar'

const en = {
  'app.name': 'TarTran',
  'nav.projects': 'Projects',
  'nav.settings': 'Settings',

  'projects.title': 'Projects',
  'projects.empty': 'No projects yet. Create your first novel project.',
  'projects.new': 'New Project',
  'projects.open': 'Open',
  'projects.delete': 'Delete',
  'projects.deleteConfirmTitle': 'Delete project?',
  'projects.deleteConfirmBody':
    'This deletes the project and all its chapters and glossary data. This cannot be undone.',
  'projects.chapters': '{count} chapters',
  'projects.translated': '{count} translated',
  'projects.reviewed': '{count} reviewed',
  'projects.updated': 'Updated {date}',
  'projects.corrupted': 'Project data could not be read',
  'projects.loadError': 'Could not load projects',

  'create.title': 'New Project',
  'create.name': 'Novel title',
  'create.namePlaceholder': 'e.g. Martial World',
  'create.author': 'Author (optional)',
  'create.target': 'Translation target language',
  'create.arabic': 'Arabic',
  'create.english': 'English',
  'create.submit': 'Create Project',
  'create.cancel': 'Cancel',
  'create.error': 'Could not create project',

  'project.back': 'Back',
  'project.notFound': 'Project not found',
  'project.chaptersTab': 'Chapters',
  'project.glossaryTab': 'Glossary',
  'project.stats': '{chapters} chapters · {translated} translated · {reviewed} reviewed',
  'project.targetTo': 'Translating to {lang}',
  'project.chaptersSoon': 'Chapter management arrives in the next milestone.',
  'project.glossarySoon': 'Glossary management arrives in the next milestone.',
  'project.deleteTitle': 'Delete project',

  'settings.title': 'Settings',
  'settings.workspace': 'Projects folder',
  'settings.workspaceHint': 'Where novel project folders are stored.',
  'settings.save': 'Save',
  'settings.saved': 'Saved',
  'settings.language': 'Interface language',
  'settings.theme': 'Theme',
  'settings.theme.system': 'System',
  'settings.theme.light': 'Light',
  'settings.theme.dark': 'Dark',
  'settings.apiKey': 'Gemini API key',
  'settings.apiKeyHint':
    'Stored encrypted on this device and never sent to the interface.',
  'settings.apiKeyConfigured': 'A key is configured.',
  'settings.apiKeyNotConfigured':
    'No key configured yet. Key management arrives with the translation milestone.',

  'common.cancel': 'Cancel',
  'common.close': 'Close',
  'common.delete': 'Delete',
  'common.loading': 'Loading…',
  'common.error': 'Something went wrong',
  'common.retry': 'Retry'
} as const

export type TKey = keyof typeof en

const ar: Record<TKey, string> = {
  'app.name': 'TarTran',
  'nav.projects': 'المشاريع',
  'nav.settings': 'الإعدادات',

  'projects.title': 'المشاريع',
  'projects.empty': 'لا توجد مشاريع بعد. أنشئ مشروع روايتك الأول.',
  'projects.new': 'مشروع جديد',
  'projects.open': 'فتح',
  'projects.delete': 'حذف',
  'projects.deleteConfirmTitle': 'حذف المشروع؟',
  'projects.deleteConfirmBody':
    'سيؤدي هذا إلى حذف المشروع بكل فصوله ومصطلحاته نهائياً. لا يمكن التراجع.',
  'projects.chapters': '{count} فصول',
  'projects.translated': '{count} مترجمة',
  'projects.reviewed': '{count} مراجعة',
  'projects.updated': 'آخر تحديث {date}',
  'projects.corrupted': 'تعذّرت قراءة بيانات المشروع',
  'projects.loadError': 'تعذّر تحميل المشاريع',

  'create.title': 'مشروع جديد',
  'create.name': 'اسم الرواية',
  'create.namePlaceholder': 'مثال: عالم الفنون القتالية',
  'create.author': 'المؤلف (اختياري)',
  'create.target': 'لغة الترجمة المستهدفة',
  'create.arabic': 'العربية',
  'create.english': 'الإنجليزية',
  'create.submit': 'إنشاء المشروع',
  'create.cancel': 'إلغاء',
  'create.error': 'تعذّر إنشاء المشروع',

  'project.back': 'رجوع',
  'project.notFound': 'المشروع غير موجود',
  'project.chaptersTab': 'الفصول',
  'project.glossaryTab': 'المصطلحات',
  'project.stats': '{chapters} فصول · {translated} مترجمة · {reviewed} مراجعة',
  'project.targetTo': 'الترجمة إلى {lang}',
  'project.chaptersSoon': 'إدارة الفصول تأتي في المرحلة القادمة.',
  'project.glossarySoon': 'إدارة المصطلحات تأتي في المرحلة القادمة.',
  'project.deleteTitle': 'حذف المشروع',

  'settings.title': 'الإعدادات',
  'settings.workspace': 'مجلد المشاريع',
  'settings.workspaceHint': 'المكان الذي تُخزَّن فيه مجلدات مشاريع الروايات.',
  'settings.save': 'حفظ',
  'settings.saved': 'تم الحفظ',
  'settings.language': 'لغة الواجهة',
  'settings.theme': 'المظهر',
  'settings.theme.system': 'تلقائي',
  'settings.theme.light': 'فاتح',
  'settings.theme.dark': 'داكن',
  'settings.apiKey': 'مفتاح Gemini API',
  'settings.apiKeyHint': 'يُخزَّن مشفراً على هذا الجهاز ولا يُرسل إلى الواجهة.',
  'settings.apiKeyConfigured': 'تم ضبط مفتاح.',
  'settings.apiKeyNotConfigured': 'لا يوجد مفتاح بعد. تأتي إدارة المفتاح مع مرحلة الترجمة.',

  'common.cancel': 'إلغاء',
  'common.close': 'إغلاق',
  'common.delete': 'حذف',
  'common.loading': 'جارٍ التحميل…',
  'common.error': 'حدث خطأ ما',
  'common.retry': 'إعادة المحاولة'
}

export const dictionaries: Record<Locale, Record<TKey, string>> = { en, ar }
export const LOCALES: Locale[] = ['en', 'ar']
