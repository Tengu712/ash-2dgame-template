import os
import platform
import shutil
import sys
from conan import ConanFile
from pathlib import Path

class Recipe(ConanFile):
	settings = "os", "compiler", "build_type", "arch"

	def requirements(self):
		self.requires("vulkan-loader/1.3.243.0")
		if os.getenv("CARGO_PROFILE") == "debug":
			self.requires("vulkan-validationlayers/1.3.243.0")

	def build_requirements(self):
		self.tool_requires("glslang/1.3.243.0")

	def configure(self):
		self.options["glslang"].hlsl = False

	def validate(self):
		if self.settings.build_type != "Release":
			raise ConanInvalidConfiguration("Only Release build is supported")

	def generate(self):
		vulkan_loader = self.dependencies["vulkan-loader"]
		vulkan_lib_dir = vulkan_loader.cpp_info.libdirs[0]
		self.output.info(f"Vulkan loader lib: {vulkan_lib_dir}")

		glslang = self.dependencies.build["glslang"]
		glslang_bin_dir = glslang.cpp_info.bindirs[0]
		self.output.info(f"glslang bin: {glslang_bin_dir}")

		vvl_json = None
		vvl_bin = None
		if os.getenv("CARGO_PROFILE") == "debug":
			vvl = self.dependencies.get("vulkan-validationlayers")
			vvl_root = Path(vvl.package_folder)
			vvl_json = self._find_path(vvl_root, "VkLayer_khronos_validation.json")
			system = platform.system()
			if system == "Windows":
				vvl_bin = self._find_path(vvl_root, "VkLayer_khronos_validation.dll")
			elif system == "Darwin":
				vvl_bin = self._find_path(vvl_root, "libVkLayer_khronos_validation.dylib")
			else:
				vvl_bin = self._find_path(vvl_root, "libVkLayer_khronos_validation.so")

		if vvl_json:
			shutil.copy2(vvl_json, os.path.join(self.recipe_folder, "deps"))
		if vvl_bin:
			shutil.copy2(vvl_bin, os.path.join(self.recipe_folder, "deps"))

		with open("conan-paths.txt", "w") as f:
			f.write(f"VULKAN_LIB={vulkan_lib_dir}\n")
			f.write(f"GLSLANG_BIN={glslang_bin_dir}\n")

	def _find_path(self, dir_path, file_name):
		founds = list(dir_path.rglob(file_name))
		if len(founds) == 0:
			self.output.error(f"{file_name} not found in: {dir_path}")
			sys.exit(1)
		if len(founds) != 1:
			self.output.error(f"multiple {file_name} found in: {dir_path}")
			for n in founds:
				self.output.info(f"  - {n}")
			sys.exit(1)
		self.output.info(f"Found: {founds[0]}")
		return founds[0]
