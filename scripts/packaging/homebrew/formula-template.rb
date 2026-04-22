# Steam Cloud File Manager Homebrew Formula 模板
# 此文件由 CI 用于生成实际的 Formula

class SteamCloudFileManager < Formula
	desc "Steam云文件管理器 - Steam Cloud File Manager"
	homepage "https://github.com/Fldicoahkiin/SteamCloudFileManager"
	version "{{VERSION}}"
	license "GPL-3.0"

	on_macos do
		if Hardware::CPU.arm?
			url(
				"https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v{{VERSION}}/SteamCloudFileManager-{{VERSION}}-macos-aarch64.tar.gz"
			)
			sha256 "{{SHA256_ARM64}}"
		else
			url(
				"https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v{{VERSION}}/SteamCloudFileManager-{{VERSION}}-macos-x86_64.tar.gz"
			)
			sha256 "{{SHA256_X64}}"
		end
	end

	def install
		app_name = "Steam Cloud File Manager.app"

		if File.exist?("Contents/Info.plist")
			(prefix / app_name).install(Dir["*"])
		else
			app_path = Dir.glob("**/*.app").first
			raise "No .app bundle found" unless app_path

			app_name = File.basename(app_path)
			prefix.install(app_path)
		end

		# 在 bin/ 目录创建启动脚本
		(bin / "steam-cloud-file-manager").write(
			<<~EOS
					#!/bin/bash
					open "#{prefix}/#{app_name}" --args "$@"
			EOS
		)
	end

	def caveats
		<<~EOS
				Steam Cloud File Manager 已安装。

				📱 **将应用添加到“应用程序”文件夹**（推荐）：
				  ln -s "#{prefix}/Steam Cloud File Manager.app" /Applications/

				🖥️ **从终端运行**：
				  steam-cloud-file-manager
		EOS
	end

	test do
		system "#{bin}/steam-cloud-file-manager", "--version"
	end
end
