import { createTRPCReact } from '@trpc/react-query';
import type { AppRouter } from '../../../api/dist/routers/index';

export const trpc = createTRPCReact<AppRouter>();