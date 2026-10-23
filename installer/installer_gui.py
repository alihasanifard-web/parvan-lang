import os
import sys
import shutil
import winreg
import subprocess
import tkinter as tk
from tkinter import ttk, messagebox

VERSION = "2.7.0"
APP_NAME = "Parvan Programming Language"
APP_NAME_FA = "زبان برنامه‌نویسی پروان"
APP_SUBTITLE = "نسخه ۲.۷.۰ — ایمپورت ماژولار، شمارشگرها و تطبیق الگو، ماشین مجازی بهینه‌شده"

DEFAULT_INSTALL_DIR = os.path.join(os.environ.get("LOCALAPPDATA", r"C:\Users\Default\AppData\Local"), "Programs", "ParvanLang")

def add_to_user_path(new_dir):
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Environment", 0, winreg.KEY_ALL_ACCESS) as key:
            try:
                cur_path, _ = winreg.QueryValueEx(key, "Path")
            except FileNotFoundError:
                cur_path = ""

            paths = [p for p in cur_path.split(";") if p.strip()]
            if new_dir not in paths:
                paths.append(new_dir)
                new_path_str = ";".join(paths)
                winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, new_path_str)
                # Broadcast environment change
                try:
                    import ctypes
                    HWND_BROADCAST = 0xFFFF
                    WM_SETTINGCHANGE = 0x001A
                    SMTO_ABORTIFHUNG = 0x0002
                    result = ctypes.c_long()
                    ctypes.windll.user32.SendMessageTimeoutW(
                        HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment",
                        SMTO_ABORTIFHUNG, 5000, ctypes.byref(result)
                    )
                except Exception:
                    pass
    except Exception as e:
        print("Path update error:", e)

def register_file_association(exe_path):
    try:
        # Register .prv file extension
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Classes\.prv") as key:
            winreg.SetValueEx(key, "", 0, winreg.REG_SZ, "ParvanLang.Script")

        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Classes\ParvanLang.Script") as key:
            winreg.SetValueEx(key, "", 0, winreg.REG_SZ, "Parvan Language Source File")

        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Classes\ParvanLang.Script\DefaultIcon") as key:
            winreg.SetValueEx(key, "", 0, winreg.REG_SZ, f'"{exe_path}",0')

        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Classes\ParvanLang.Script\shell\open\command") as key:
            winreg.SetValueEx(key, "", 0, winreg.REG_SZ, f'"{exe_path}" "%1"')
    except Exception as e:
        print("File association error:", e)

def register_uninstaller(install_dir, exe_path):
    try:
        uninstall_cmd = os.path.join(install_dir, "uninstall.cmd")
        with open(uninstall_cmd, "w", encoding="utf-8") as f:
            f.write(f'''@echo off
title Uninstall ParvanLang
echo Removing Parvan Programming Language...
reg delete "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ParvanLang" /f >nul 2>&1
reg delete "HKCU\\Software\\Classes\\.prv" /f >nul 2>&1
reg delete "HKCU\\Software\\Classes\\ParvanLang.Script" /f >nul 2>&1
del "%USERPROFILE%\\Desktop\\Parvan REPL.lnk" >nul 2>&1
rmdir /s /q "{install_dir}"
echo ParvanLang was removed successfully.
pause
''')

        key_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\ParvanLang"
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path) as key:
            winreg.SetValueEx(key, "DisplayName", 0, winreg.REG_SZ, f"{APP_NAME_FA} (ParvanLang v{VERSION})")
            winreg.SetValueEx(key, "DisplayVersion", 0, winreg.REG_SZ, VERSION)
            winreg.SetValueEx(key, "Publisher", 0, winreg.REG_SZ, "Parvan Team (Ali Hasanifard, Ehsan Fazlinejad, Amirmehdi Shahbazi)")
            winreg.SetValueEx(key, "InstallLocation", 0, winreg.REG_SZ, install_dir)
            winreg.SetValueEx(key, "DisplayIcon", 0, winreg.REG_SZ, exe_path)
            winreg.SetValueEx(key, "UninstallString", 0, winreg.REG_SZ, f'"{uninstall_cmd}"')
    except Exception as e:
        print("Uninstaller registry error:", e)


class InstallerGUI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title(f"ParvanLang v{VERSION} Setup Wizard")
        self.geometry("680x520")
        self.resizable(False, False)
        self.configure(bg="#05070d")

        # Center window
        self.update_idletasks()
        w = self.winfo_width()
        h = self.winfo_height()
        x = (self.winfo_screenwidth() // 2) - (w // 2)
        y = (self.winfo_screenheight() // 2) - (h // 2)
        self.geometry(f"{w}x{h}+{x}+{y}")

        self.setup_ui()

    def setup_ui(self):
        # 1. SLEEK MINIMAL HEADER (DEEP OBSIDIAN #030407)
        header = tk.Frame(self, bg="#030407", height=100)
        header.pack(fill="x")
        header.pack_propagate(False)

        header_border = tk.Frame(self, bg="#111827", height=1)
        header_border.pack(fill="x")

        logo_frame = tk.Frame(header, bg="#030407")
        logo_frame.pack(fill="both", expand=True, padx=28)

        # Load transparent logo
        self.logo_img = None
        logo_path = os.path.join(getattr(sys, '_MEIPASS', os.path.dirname(__file__)), "setup_logo.png")
        if os.path.exists(logo_path):
            try:
                from PIL import Image, ImageTk
                raw_img = Image.open(logo_path).resize((68, 68), Image.Resampling.LANCZOS)
                self.logo_img = ImageTk.PhotoImage(raw_img)
                lbl_icon = tk.Label(logo_frame, image=self.logo_img, bg="#030407")
                lbl_icon.pack(side="right", padx=10, pady=16)
            except Exception:
                pass

        text_box = tk.Frame(logo_frame, bg="#030407")
        text_box.pack(side="left", fill="y", pady=22)

        badge_frame = tk.Frame(text_box, bg="#080c14", padx=7, pady=2, highlightthickness=1, highlightbackground="#1e293b")
        badge_frame.pack(anchor="w")
        tk.Label(badge_frame, text="PRODUCTION RELEASE", font=("Consolas", 8, "bold"), fg="#38bdf8", bg="#080c14").pack()

        title_lbl = tk.Label(text_box, text=f"{APP_NAME_FA} (ParvanLang)", font=("Vazirmatn", 15, "bold"), fg="#f8fafc", bg="#030407")
        title_lbl.pack(anchor="w", pady=(4, 0))

        sub_lbl = tk.Label(text_box, text=APP_SUBTITLE, font=("Vazirmatn", 8), fg="#64748b", bg="#030407")
        sub_lbl.pack(anchor="w")

        # 2. MAIN BODY (DEEP PURE DARK #030407)
        body = tk.Frame(self, bg="#030407", padx=32, pady=14)
        body.pack(fill="both", expand=True)

        # Feature pills row (Deep minimal cards)
        pills_frame = tk.Frame(body, bg="#030407")
        pills_frame.pack(fill="x", pady=(0, 12))

        pills = [
            ("⚡ Rust Native VM", "#38bdf8"),
            ("📦 Standalone Builder", "#a855f7"),
            ("🔮 LSP Protocol Server", "#10b981"),
            ("🗄️ Embedded Database", "#f59e0b"),
        ]
        for tag, color in pills:
            p_box = tk.Frame(pills_frame, bg="#070a12", padx=9, pady=3, highlightthickness=1, highlightbackground="#131b2e")
            p_box.pack(side="right", padx=3)
            tk.Label(p_box, text=tag, font=("Vazirmatn", 8), fg=color, bg="#070a12").pack()

        # Destination Folder Card (Subtle deep surface)
        dest_card = tk.Frame(body, bg="#070a12", padx=16, pady=10, highlightthickness=1, highlightbackground="#131b2e")
        dest_card.pack(fill="x", pady=6)

        tk.Label(dest_card, text="مسیر نصب موتور اجرایی و ابزارهای پروان:", font=("Vazirmatn", 9), fg="#64748b", bg="#070a12").pack(anchor="e")
        path_box = tk.Label(dest_card, text=DEFAULT_INSTALL_DIR, font=("Consolas", 9), fg="#38bdf8", bg="#030407", padx=10, pady=4, anchor="w")
        path_box.pack(fill="x", pady=(4, 0))

        # Checkboxes Container Card (Clean dark surface)
        opt_card = tk.Frame(body, bg="#070a12", padx=16, pady=10, highlightthickness=1, highlightbackground="#131b2e")
        opt_card.pack(fill="x", pady=6)

        self.opt_path = tk.BooleanVar(value=True)
        self.opt_assoc = tk.BooleanVar(value=True)

        cb1 = tk.Checkbutton(opt_card, text="افزودن دستور parvan به متغیرهای محیطی سیستم (PATH)",
                             variable=self.opt_path, font=("Vazirmatn", 9), fg="#cbd5e1", bg="#070a12",
                             selectcolor="#030407", activebackground="#070a12", activeforeground="#38bdf8")
        cb1.pack(anchor="e", pady=3)

        cb2 = tk.Checkbutton(opt_card, text="اتصال و رجیستر پسوند فایل‌های .prv برای اجرای مستقیم",
                             variable=self.opt_assoc, font=("Vazirmatn", 9), fg="#cbd5e1", bg="#070a12",
                             selectcolor="#030407", activebackground="#070a12", activeforeground="#38bdf8")
        cb2.pack(anchor="e", pady=3)

        # Sleek Minimal Progress Bar
        style = ttk.Style()
        style.theme_use('clam')
        style.configure("Cyber.Horizontal.TProgressbar",
                        troughcolor="#030407",
                        background="#0284c7",
                        lightcolor="#38bdf8",
                        darkcolor="#0369a1",
                        bordercolor="#111827",
                        thickness=6)

        self.progress = ttk.Progressbar(body, orient="horizontal", style="Cyber.Horizontal.TProgressbar", mode="determinate")
        self.progress.pack(fill="x", pady=(10, 4))

        self.status_lbl = tk.Label(body, text="آماده استقرار و راه‌اندازی اکوسیستم پروان...", font=("Vazirmatn", 9), fg="#475569", bg="#030407")
        self.status_lbl.pack(anchor="center")

        # 3. MINIMAL ACTION FOOTER (PURE DARK)
        footer_border = tk.Frame(self, bg="#111827", height=1)
        footer_border.pack(fill="x", side="bottom")

        footer = tk.Frame(self, bg="#030407", height=66, padx=28)
        footer.pack(fill="x", side="bottom")
        footer.pack_propagate(False)

        author_lbl = tk.Label(footer, text="Parvan Core Team © 2026", font=("Consolas", 8), fg="#334155", bg="#030407")
        author_lbl.pack(side="left", pady=22)

        btn_cluster = tk.Frame(footer, bg="#030407")
        btn_cluster.pack(side="right", pady=14)

        self.close_btn = tk.Button(btn_cluster, text="انصراف", font=("Vazirmatn", 10),
                                   bg="#070a12", fg="#64748b", activebackground="#0f172a", activeforeground="#f8fafc",
                                   padx=16, pady=4, relief="flat", borderwidth=1, highlightbackground="#1e293b",
                                   cursor="hand2", command=self.destroy)
        self.close_btn.pack(side="right", padx=8)

        self.install_btn = tk.Button(btn_cluster, text="نصب پروان (Install)", font=("Vazirmatn", 10, "bold"),
                                     bg="#0284c7", fg="#ffffff", activebackground="#38bdf8", activeforeground="#000000",
                                     padx=22, pady=4, relief="flat", cursor="hand2", command=self.start_install)
        self.install_btn.pack(side="right")

    def start_install(self):
        self.install_btn.config(state="disabled", bg="#1e293b", fg="#64748b")
        self.close_btn.config(state="disabled")

        src_exe = r"C:\Users\V I S I O N\Desktop\parvan.exe"
        dest_dir = DEFAULT_INSTALL_DIR
        dest_exe = os.path.join(dest_dir, "parvan.exe")

        try:
            self.status_lbl.config(text="در حال پیکربندی و آماده‌سازی مخزن برنامه...", fg="#38bdf8")
            self.progress["value"] = 25
            self.update()

            os.makedirs(dest_dir, exist_ok=True)
            self.status_lbl.config(text="در حال استقرار باینری مفسر و ماشین مجازی بایت‌کد...", fg="#38bdf8")
            self.progress["value"] = 55
            self.update()

            if os.path.exists(src_exe):
                shutil.copy2(src_exe, dest_exe)

            if self.opt_path.get():
                self.status_lbl.config(text="در حال ثبت دستور parvan در PATH سیستم...", fg="#c084fc")
                self.progress["value"] = 75
                self.update()
                add_to_user_path(dest_dir)

            if self.opt_assoc.get():
                self.status_lbl.config(text="در حال ثبت نوع فایل و پیوند پسوند prv. در رجیستری...", fg="#c084fc")
                self.progress["value"] = 90
                self.update()
                register_file_association(dest_exe)

            register_uninstaller(dest_dir, dest_exe)

            self.progress["value"] = 100
            self.status_lbl.config(text="✔ فرآیند نصب با موفقیت کامل شد!", fg="#34d399")
            self.update()

            messagebox.showinfo("نصب موفقیت‌آمیز",
                                f"اکوسیستم زبان برنامه‌نویسی پروان با موفقیت در مسیر زیر مستقر گردید:\n\n"
                                f"{dest_dir}\n\n"
                                f"دستور 'parvan' اکنون در تمامی ترمینال‌ها (cmd، PowerShell و bash) در دسترس است.\n"
                                f"فایل‌های با پسوند prv. نیز با یک کلیک اجرا می‌شوند.")
            self.destroy()
        except Exception as e:
            messagebox.showerror("خطای نصب", f"خطایی در هنگام فرآیند نصب رخ داد:\n{e}")
            self.install_btn.config(state="normal", bg="#38bdf8", fg="#05070d")
            self.close_btn.config(state="normal")

if __name__ == "__main__":
    app = InstallerGUI()
    app.mainloop()
