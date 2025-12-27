@echo off

cd /d "%~dp0"

cl /c /O2 /MD /EHsc /utf-8 window.cpp
lib /OUT:..\..\deps\window.lib window.obj

del window.obj
