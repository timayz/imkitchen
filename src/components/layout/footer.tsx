'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import { Heart, Github, Twitter, Mail } from 'lucide-react';

export function Footer() {
  const pathname = usePathname();
  const t = useTranslations();

  // Extract locale from pathname
  const locale = pathname.split('/')[1] || 'en';

  const currentYear = new Date().getFullYear();

  return (
    <footer className="mt-auto bg-white border-t border-gray-200">
      <div className="container mx-auto px-4 py-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Brand */}
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="h-8 w-8 rounded-lg bg-orange-500 flex items-center justify-center">
                <span className="text-white font-bold text-lg">iK</span>
              </div>
              <span className="text-xl font-bold text-gray-900">imkitchen</span>
            </div>
            <p className="text-gray-600 text-sm">{t('footer.description')}</p>
          </div>

          {/* Legal Links */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
              {t('footer.legal')}
            </h3>
            <ul className="space-y-2">
              <li>
                <Link
                  href={`/${locale}/privacy`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.privacy')}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/terms`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.terms')}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/cookies`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.cookies')}
                </Link>
              </li>
            </ul>
          </div>

          {/* Support */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
              {t('footer.support')}
            </h3>
            <ul className="space-y-2">
              <li>
                <Link
                  href={`/${locale}/help`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.help')}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/contact`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.contact')}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/faq`}
                  className="text-gray-600 hover:text-gray-900 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded"
                >
                  {t('footer.faq')}
                </Link>
              </li>
            </ul>
          </div>

          {/* Social Media */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">
              {t('footer.connect')}
            </h3>
            <div className="flex space-x-4">
              <a
                href="https://github.com/imkitchen"
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-600 hover:text-gray-900 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded p-1"
                aria-label={t('footer.github')}
                style={{ minHeight: '44px', minWidth: '44px' }}
              >
                <Github className="h-6 w-6" />
              </a>
              <a
                href="https://twitter.com/imkitchen"
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-600 hover:text-gray-900 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded p-1"
                aria-label={t('footer.twitter')}
                style={{ minHeight: '44px', minWidth: '44px' }}
              >
                <Twitter className="h-6 w-6" />
              </a>
              <a
                href="mailto:support@imkitchen.com"
                className="text-gray-600 hover:text-gray-900 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 rounded p-1"
                aria-label={t('footer.email')}
                style={{ minHeight: '44px', minWidth: '44px' }}
              >
                <Mail className="h-6 w-6" />
              </a>
            </div>
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="mt-8 pt-8 border-t border-gray-200">
          <div className="flex flex-col md:flex-row justify-between items-center space-y-4 md:space-y-0">
            <p className="text-gray-600 text-sm">
              © {currentYear} imkitchen. {t('footer.rights')}
            </p>
            <div className="flex items-center space-x-2 text-sm text-gray-600">
              <span>{t('footer.madeWith')}</span>
              <Heart className="h-4 w-4 text-red-500" />
              <span>{t('footer.forCooks')}</span>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
