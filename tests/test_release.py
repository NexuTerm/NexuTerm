"""Release dependency regressions; no network, root privileges or remote servers."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "nexuterm_release", Path(__file__).resolve().parents[1] / "packaging/release.py")
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def test_vnc_has_no_external_viewer_dependency(self):
        dependencies = release.package_dependencies("libvncclient1 (>= 0.9.14)")
        self.assertIn("libvncclient1 (>= 0.9.14)", dependencies)
        self.assertNotIn("tigervnc", dependencies)
        self.assertNotIn("remmina", dependencies)
        self.assertNotIn("tnftp", dependencies)
        self.assertNotIn("freerdp", dependencies)
        self.assertNotIn("RDP", [protocol for protocol, _, _ in release.CLIENTS])
        self.assertNotIn("VNC", [protocol for protocol, _, _ in release.CLIENTS])

    def test_dependencies_include_external_clients_and_api_minimums(self):
        dependencies = release.package_dependencies(
            "libc6 (>= 2.39), libgtk-4-1 (>= 4.9.2), "
            "libadwaita-1-0 (>= 1.4~beta), libvte-2.91-gtk4-0 (>= 0.49.92)")
        for required in ("openssh-client", "inetutils-telnet", "curl", "gvfs-backends", "gnome-keyring | keepassxc",
                         "libc6 (>= 2.39)",
                         "libgtk-4-1 (>= 4.10)", "libadwaita-1-0 (>= 1.4)",
                         "libvte-2.91-gtk4-0 (>= 0.70)"):
            self.assertIn(required, dependencies.split(", "))

    def test_newer_detected_library_requirement_is_preserved(self):
        dependencies = release.package_dependencies("libgtk-4-1 (>= 4.22)")
        self.assertIn("libgtk-4-1 (>= 4.22)", dependencies)
        self.assertNotIn("libgtk-4-1 (>= 4.10)", dependencies)

    def test_runtime_check_rejects_missing_viewers_and_libraries(self):
        with tempfile.NamedTemporaryFile() as binary:
            with patch.object(release, "find_client", return_value=None), \
                    patch.object(release, "find_ftp_backend", return_value=None), \
                    patch.object(release, "output", return_value="libgtk-4.so.1 => not found"):
                with self.assertRaises(RuntimeError) as error:
                    release.check_runtime(Path(binary.name))
            for missing in ("openssh-client", "inetutils-telnet", "curl", "gvfs-backends", "not found"):
                self.assertIn(missing, str(error.exception))

    def test_client_lookup_accepts_alias_and_ignores_relative_path(self):
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / "xfreerdp"
            executable.write_text("test fixture, never executed")
            executable.chmod(0o755)
            with patch.object(release.os, "get_exec_path", return_value=[directory]):
                self.assertEqual(release.find_client(("xfreerdp3", "xfreerdp")), executable)
            with patch.object(release.os, "get_exec_path", return_value=["."]):
                self.assertIsNone(release.find_client(("xfreerdp3", "xfreerdp")))


if __name__ == "__main__":
    unittest.main()
