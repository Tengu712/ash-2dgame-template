CMAKE_MINIMUM_REQUIRED(VERSION 3.31.0)

# Script modeでの実行を強要
#
# NOTE: このファイルは依存パッケージのビルドをRustで書くよりも簡単に書くためのスクリプトなので、
#       CMakeのビルド機能は一切必要としていない。
# NOTE: 誤って`cmake .`のように実行するとCMakeFilesとCMakeCache.txtが生成される。
if(NOT CMAKE_SCRIPT_MODE_FILE)
	message(FATAL_ERROR "You must run this file in script mode. Run with: cmake -P install_deps.cmake")
endif()

# 便利定数を定義
set(VK_VER_SHORT "1.4.335")
set(VK_VER       "vulkan-sdk-${VK_VER_SHORT}.0")
set(DEPS_DIR     "${CMAKE_CURRENT_LIST_DIR}/deps")

# 取り敢えずdepsディレクトリを作成
file(MAKE_DIRECTORY "${DEPS_DIR}")

# 外部コマンド実行マクロ
#
# 最後の引数に実行ディレクトリとしてDEPS_DIRからの相対パスを与える。
macro(run)
	set(run_args ${ARGV})
	list(GET       run_args -1 wd)
	list(REMOVE_AT run_args -1)
	set(cmd ${run_args})

	execute_process(
		COMMAND           ${cmd}
		WORKING_DIRECTORY "${DEPS_DIR}/${wd}"
		RESULT_VARIABLE   res
	)
	if(NOT ${res} EQUAL 0)
		message(FATAL_ERROR "Command failed with exit code: ${res}")
	endif()
endmacro()

# Vulkanライブラリインストール関数
#
# 最初の引数にライブラリ名を与える。
# 残りの引数はcmake config時の引数に渡される。
function(install_dep)
	set(args ${ARGV})
	list(GET       args 0 name)
	list(REMOVE_AT args 0)

	set(${name}_SOURCE_DIR  "${DEPS_DIR}/${name}-${VK_VER}"         PARENT_SCOPE)
	set(${name}_INSTALL_DIR "${DEPS_DIR}/${name}-${VK_VER}/install" PARENT_SCOPE)

	if(EXISTS "${DEPS_DIR}/${name}-${VK_VER}/install/.stamp")
		return()
	endif()

	if(NOT EXISTS "${DEPS_DIR}/${name}-${VK_VER}")
		run(curl -4 -L -O https://github.com/KhronosGroup/${name}/archive/refs/tags/${VK_VER}.tar.gz ".")
		run(tar xzf ${VK_VER}.tar.gz                                                                 ".")
	endif()

	run(cmake -Bbuild -GNinja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=install ${args} ${name}-${VK_VER})
	run(cmake --build   build                                                                   ${name}-${VK_VER})
	run(cmake --install build                                                                   ${name}-${VK_VER})

	file(TOUCH  "${DEPS_DIR}/${name}-${VK_VER}/install/.stamp")
	file(REMOVE "${DEPS_DIR}/${VK_VER}.tar.gz")
endfunction()

# 必須Vulkanライブラリをインストール
install_dep(Vulkan-Headers)
install_dep(
	Vulkan-Loader
	"-DVULKAN_HEADERS_INSTALL_DIR=${Vulkan-Headers_INSTALL_DIR}"
)
install_dep(SPIRV-Headers)
install_dep(
	SPIRV-Tools
	"-DSPIRV-Headers_SOURCE_DIR=${SPIRV-Headers_SOURCE_DIR}"
	-DSPIRV_SKIP_TESTS=ON
	-DSPIRV_SKIP_EXECUTABLES=ON
	-DBUILD_SHARED_LIBS=OFF
)
install_dep(
	glslang
	"-DCMAKE_PREFIX_PATH=${SPIRV-Tools_INSTALL_DIR}"
	-DALLOW_EXTERNAL_SPIRV_TOOLS=ON
)

# VVL有効時は次を実行
# - VVLをインストール
# - depsにコピー
if(VVL)
	install_dep(
		Vulkan-Utility-Libraries
		"-DVULKAN_HEADERS_INSTALL_DIR=${Vulkan-Headers_INSTALL_DIR}"
	)

	if(UNIX AND NOT APPLE)
		set(add_flags -DBUILD_WSI_XLIB_SUPPORT=OFF -DBUILD_WSI_WAYLAND_SUPPORT=OFF)
	else()
		set(add_flags "")
	endif()
	install_dep(
		Vulkan-ValidationLayers
		"-DVULKAN_HEADERS_INSTALL_DIR=${Vulkan-Headers_INSTALL_DIR}"
		"-DSPIRV_HEADERS_INSTALL_DIR=${SPIRV-Headers_INSTALL_DIR}"
		"-DSPIRV_TOOLS_INSTALL_DIR=${SPIRV-Tools_INSTALL_DIR}"
		"-DGLSLANG_INSTALL_DIR=${glslang_INSTALL_DIR}"
		"-DVULKAN_UTILITY_LIBRARIES_INSTALL_DIR=${Vulkan-Utility-Libraries_INSTALL_DIR}"
		${add_flags}
	)

	if(WIN32)
		set(name "VkLayer_khronos_validation.dll")
		set(lib  "${Vulkan-ValidationLayers_INSTALL_DIR}/bin/${name}")
		set(json "${Vulkan-ValidationLayers_INSTALL_DIR}/bin/VkLayer_khronos_validation.json")
	elseif(APPLE)
		set(name "libVkLayer_khronos_validation.dylib")
		set(lib  "${Vulkan-ValidationLayers_INSTALL_DIR}/lib/${name}")
		set(json "${Vulkan-ValidationLayers_INSTALL_DIR}/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json")
	else()
		set(name "libVkLayer_khronos_validation.so")
		set(lib  "${Vulkan-ValidationLayers_INSTALL_DIR}/lib/${name}")
		set(json "${Vulkan-ValidationLayers_INSTALL_DIR}/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json")
	endif()
	if(NOT EXISTS "${DEPS_DIR}/${name}")
		file(COPY "${lib}"  DESTINATION "${DEPS_DIR}")
	endif()
	if(NOT EXISTS "${DEPS_DIR}/VkLayer_khronos_validation.json")
		file(COPY "${json}" DESTINATION "${DEPS_DIR}")
	endif()
endif()

# macOSは以下を実行
# - MoltenVKをダウンロード
# - MoltenVKをCARGO_BUILD_PATHにコピー
# - Vulkan LoaderをCARGO_BUILD_PATHにコピー
if(APPLE)
	if(NOT CARGO_BUILD_PATH)
		message(FATAL_ERROR "CARGO_BUILD_PATH not defined.")
	endif()

	if(NOT EXISTS "${DEPS_DIR}/MoltenVK")
		run(curl -4 -L -O https://github.com/KhronosGroup/MoltenVK/releases/download/v1.4.1/MoltenVK-macos.tar ".")
		run(tar xf MoltenVK-macos.tar                                                                          ".")
	endif()

	if(NOT EXISTS "${CARGO_BUILD_PATH}/libMoltenVK.dylib")
		file(COPY "${DEPS_DIR}/MoltenVK/MoltenVK/dynamic/dylib/macOS/libMoltenVK.dylib" DESTINATION "${CARGO_BUILD_PATH}")
	endif()
	if(NOT EXISTS "${CARGO_BUILD_PATH}/MoltenVK_icd.json")
		file(COPY "${DEPS_DIR}/MoltenVK/MoltenVK/dynamic/dylib/macOS/MoltenVK_icd.json" DESTINATION "${CARGO_BUILD_PATH}")
	endif()

	if(NOT EXISTS "${CARGO_BUILD_PATH}/libvulkan.1.dylib")
		file(COPY   "${Vulkan-Loader_INSTALL_DIR}/lib/libvulkan.${VK_VER_SHORT}.dylib" DESTINATION "${CARGO_BUILD_PATH}")
		file(RENAME "${CARGO_BUILD_PATH}/libvulkan.${VK_VER_SHORT}.dylib" "${CARGO_BUILD_PATH}/libvulkan.1.dylib")
	endif()
endif()

# ウィンドウライブラリをビルド
if(WIN32 AND NOT EXISTS "${DEPS_DIR}/window.lib")
	run("${DEPS_DIR}/../window/windows/build.bat" ".")
elseif(APPLE AND NOT EXISTS "${DEPS_DIR}/libwindow.a")
	run("${DEPS_DIR}/../window/macos/build.sh" ".")
elseif(UNIX AND NOT APPLE AND NOT EXISTS "${DEPS_DIR}/libwindow.a")
	run("${DEPS_DIR}/../window/linux/build.sh" ".")
endif()
