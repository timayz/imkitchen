require 'json'

package = JSON.parse(File.read(File.join(__dir__, '..', 'package.json')))

# iOS side of `sparkling-keep-awake` (see ../README.md). Linked into the app
# by `sparkling autolink` through the `ios` section of ../module.config.json.
Pod::Spec.new do |s|
  s.name           = 'Sparkling-KeepAwake'
  s.version        = package['version']
  s.summary        = package['description']
  s.description    = package['description']
  s.license        = { :type => 'Proprietary', :text => 'imkitchen' }
  s.author         = 'imkitchen'
  s.homepage       = 'https://imkitchen.app'
  s.platforms      = { :ios => '12.0' }
  s.swift_version  = '5.7'
  s.source         = { :path => '.' }
  s.static_framework = true

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule'
  }

  s.source_files = 'Sources/**/*.swift'
  s.frameworks   = 'UIKit'

  s.dependency 'SparklingMethod/Core'
end
