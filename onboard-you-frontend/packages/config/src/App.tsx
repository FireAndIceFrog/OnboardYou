import { RouterProvider } from 'react-router-dom';
import { router } from '@/router';
import '@/styles/config.scss';
import '@xyflow/react/dist/style.css';

export default function App() {
  return <RouterProvider router={router} />;
}
