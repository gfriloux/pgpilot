import { BrowserRouter, Routes, Route } from 'react-router-dom';
import AppLayout from './layout/AppLayout';
import MyKeys from './pages/MyKeys';
import PublicKeys from './pages/PublicKeys';
import CreateKey from './pages/CreateKey';
import Import from './pages/Import';
import Encrypt from './pages/Encrypt';
import Decrypt from './pages/Decrypt';
import Sign from './pages/Sign';
import Verify from './pages/Verify';
import Chat from './pages/Chat';
import Health from './pages/Health';
import Settings from './pages/Settings';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route index element={<MyKeys />} />
          <Route path="public-keys" element={<PublicKeys />} />
          <Route path="create-key" element={<CreateKey />} />
          <Route path="import" element={<Import />} />
          <Route path="encrypt" element={<Encrypt />} />
          <Route path="decrypt" element={<Decrypt />} />
          <Route path="sign" element={<Sign />} />
          <Route path="verify" element={<Verify />} />
          <Route path="chat" element={<Chat />} />
          <Route path="health" element={<Health />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
