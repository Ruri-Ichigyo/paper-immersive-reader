import { Routes, Route, Navigate } from "react-router-dom";
import AppLayout from "./components/AppLayout";
import LibraryPage from "./pages/LibraryPage";
import ImportPage from "./pages/ImportPage";
import ReadPage from "./pages/ReadPage";
import VocabularyPage from "./pages/VocabularyPage";
import AnnotationsPage from "./pages/AnnotationsPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<Navigate to="/library" replace />} />
        <Route path="/library" element={<LibraryPage />} />
        <Route path="/import" element={<ImportPage />} />
        <Route path="/read/:paperId" element={<ReadPage />} />
        <Route path="/vocabulary" element={<VocabularyPage />} />
        <Route path="/annotations" element={<AnnotationsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  );
}