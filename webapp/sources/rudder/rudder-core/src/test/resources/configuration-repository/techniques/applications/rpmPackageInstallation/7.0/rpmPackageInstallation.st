#####################################################################################
# Copyright 2011-2013 Normation SAS
#####################################################################################
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, Version 3.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
#####################################################################################

##########################################################################
# Installs requested packages by OS                                      #
#                                                                        #
# - RPM_PACKAGE_REDLIST --> The OS dependant packages to install         #
#                                                                        #
# - RPM_PACKAGE_REDACTION -->  The action to be made on these packages   #
#                                                                        #
# - RPM_PACKAGE_VERSION -->  The optionnal package version               #
#                                                                        #
##########################################################################

bundle agent check_rpm_package_installation {

  vars:
      &RPM_PACKAGE_CHECK_INTERVAL:{check_interval |"rpm_package_check_interval" string => "&check_interval&";
      }&

      "package_number" int => readstringarrayidx("rpm_data","${this.promise_dirname}/rpmPackageInstallationData", "#[^\n]*",":",9000,1600000);
&! We need to use the i0 notation to have indexes starting at 0, as in readstringarrayidx !&

      &RPM_PACKAGE_REDLIST:{name |"rpm_package[&i0&]" string => "&name&";
}&

      &RPM_PACKAGE_VERSION:{version |"rpm_version[&i0&]" string => "&version&";
}&

      &RPM_PACKAGE_POST_HOOK_COMMAND:{command |"rpm_posthook[&i0&]" string => "&command&";
}&

      "index_rpmpkg" slist => getindices("rpm_package");

    # File containing the list of available packages to install
      "available_updates_file" string => "${sys.workdir}/state/software_available_updates.csv";

    redhat::
      "available_updates_command" string => "/usr/bin/yum ${redhat_knowledge.yum_options} check-update | ${paths.path[sed]} 's%[0-9]\+:%%g' > ${available_updates_file}";
    zypper_version_ok::
      "available_updates_command" string => "/usr/bin/zypper --non-interactive list-updates -a | ${paths.path[grep]} '|' | ${paths.path[awk]} -F '|' '{print $3 $5}' | ${paths.path[sed]} 's%[0-9]\+:%%g' > ${available_updates_file}";

    # If the promises have been updated, we need to remove previously
    # defined persistent classes (by setting there persistence to 1 minutes)
    # and cancelling also the classes for this run
    rudder_promises_generated_repaired.!pass1.!pass2.!pass3::
      "package_classes_to_cancel_suffix" slist => { "kept", "repaired", "error", "failed", "denied", "timeout" }; # all classes set by rudder_common_classes_persist body
      "zmd_classes_to_cancel_on_update" slist => { "zmd_kept", "zmd_restarted", "could_not_restart_zmd" };

      "unpersist_available_packages_classes_${package_classes_to_cancel_suffix}"
        string  => "undefine",
        classes => rudder_always_classes_persist("rpm_package_installation_updated_available_packages_${package_classes_to_cancel_suffix}", "1");

      "available_packages_classes_purged_${package_classes_to_cancel_suffix}"
        string  => "undefine",
        classes => cancel_all_classes("rpm_package_installation_updated_available_packages_${package_classes_to_cancel_suffix}");

      "unpersist_rpm_classes_${index_rpmpkg}_${package_classes_to_cancel_suffix}"
        string  => "undefine",
        classes => rudder_always_classes_persist("rpm_package_installation_${index_rpmpkg}_${package_classes_to_cancel_suffix}", "1");

      "rpm_classes_purged_${index_rpmpkg}_${package_classes_to_cancel_suffix}"
        string  => "undefine",
        classes => cancel_all_classes("rpm_package_installation_${index_rpmpkg}_${package_classes_to_cancel_suffix}");

      "unpersist_${zmd_classes_to_cancel_on_update}"
        string  => "undefine",
        classes => rudder_always_classes_persist("${zmd_classes_to_cancel_on_update}", "1");

      "${zmd_classes_to_cancel_on_update}_purged"
        string  => "undefine",
        classes => cancel_all_classes("${zmd_classes_to_cancel_on_update}");

      # Generate the list of avalable package if it has been done more than xx minutes ago, or if the file is not there
      # Do it on redhat, or if the version of zypper is valid
      (redhat|zypper_version_ok).first_pass::
        "get_update_list" string => execresult("${available_updates_command}", "useshell"),
          classes => rudder_common_classes_persist("rpm_package_installation_updated_available_packages", "${rpm_package_check_interval}"),
          ifvarclass => "!(rpm_package_installation_updated_available_packages_repaired|rpm_package_installation_updated_available_packages_kept)|!update_file_exists";

      # If the update file exist, then gather the version of the packages in it
      update_file_exists::
        "${index_rpmpkg}_update_version" string => execresult("${paths.path[grep]} \"${rpm_package[${index_rpmpkg}]}.\" ${available_updates_file} | ${paths.path[awk]} 'END {print $2}'", "useshell"),
          classes => rudder_common_classes("${index_rpmpkg}_update_version_fetched"),
          ifvarclass => "!(${index_rpmpkg}_update_version_fetched_kept|${index_rpmpkg}_update_version_fetched_repaired|${index_rpmpkg}_update_version_fetched_failed)";


  classes:
      &RPM_PACKAGE_POST_HOOK_RUN:{run |"execute_command_&i0&" expression => strcmp("&run&", "true");
}&

      # Check if update file exists
      "update_file_exists" expression => fileexists("${available_updates_file}");

      # Detect if we want to update the package
      "is_update_action_${index_rpmpkg}" expression => strcmp("${rpm_data[${index_rpmpkg}][0]}", "update");

      # Gather the version of the available package
      "${index_rpmpkg}_update_version_valid" not => strcmp("", "${${index_rpmpkg}_update_version}"),
        ifvarclass => "(${index_rpmpkg}_update_version_fetched_kept|${index_rpmpkg}_update_version_fetched_repaired)";

      "${index_rpmpkg}_package_version_defined" not => strcmp("${rpm_data[${index_rpmpkg}][1]}", "default");

      "first_pass" expression => "any";

    any::

      "pass3" expression => "pass2";
      "pass2" expression => "pass1";
      "pass1" expression => "any";

  packages:

    redhat|SuSE::

      # Case 1: Not an update, or an update on SuSE without proper Zypper
      "${rpm_package[${index_rpmpkg}]}"
        package_policy  => "${rpm_data[${index_rpmpkg}][0]}",
        package_method  => generic_nobulk("30"),
        classes => rudder_common_classes_persist("rpm_package_installation_${index_rpmpkg}", "${rpm_package_check_interval}"),
        ifvarclass => "(!is_update_action_${index_rpmpkg}|(SuSE.!zypper_version_ok)).!${index_rpmpkg}_package_version_defined.!(rpm_package_installation_${index_rpmpkg}_repaired|rpm_package_installation_${index_rpmpkg}_error|rpm_package_installation_${index_rpmpkg}_kept)",
        comment => "Handling ${rpm_package[${index_rpmpkg}]} using yum_nobulk, policy : ${rpm_data[${index_rpmpkg}][0]}";

      # Case 2: It is an update, on RedHat or SuSE with Zypper, and we didn't give a version, but we could fetch it
      "${rpm_package[${index_rpmpkg}]}"
        package_policy  => "${rpm_data[${index_rpmpkg}][0]}",
        package_method  => generic_nobulk("30"),
        package_select  => "==",
        package_version => "${${index_rpmpkg}_update_version}",
        classes         => rudder_common_classes_persist("rpm_package_installation_${index_rpmpkg}", "${rpm_package_check_interval}"),
        ifvarclass      => "is_update_action_${index_rpmpkg}.${index_rpmpkg}_update_version_valid.!${index_rpmpkg}_package_version_defined.!(rpm_package_installation_${index_rpmpkg}_repaired|rpm_package_installation_${index_rpmpkg}_error|rpm_package_installation_${index_rpmpkg}_kept)",
        comment         => "Handling ${rpm_package[${index_rpmpkg}]} using yum_nobulk, policy : ${rpm_data[${index_rpmpkg}][0]}";

      # Case 3: We do have a version defined
      "${rpm_package[${index_rpmpkg}]}"
        package_policy   => "${rpm_data[${index_rpmpkg}][0]}",
        package_method   => generic_nobulk_version("30"),
        package_select   => "${rpm_data[${index_rpmpkg}][2]}",
        package_version  => "${rpm_version[${index_rpmpkg}]}",
        classes => rudder_common_classes_persist("rpm_package_installation_${index_rpmpkg}", "${rpm_package_check_interval}"),
        ifvarclass => "${index_rpmpkg}_package_version_defined.!(rpm_package_installation_${index_rpmpkg}_repaired|rpm_package_installation_${index_rpmpkg}_error|rpm_package_installation_${index_rpmpkg}_kept)",
        comment => "Handling ${rpm_package[${index_rpmpkg}]} using yum_nobulk, policy : ${rpm_data[${index_rpmpkg}][0]}";

  methods:

    pass2.zmd_restart.!zypper_version_ok::

      "restart_zmd" usebundle => service_restart("novell-zmd"),
                      classes => kept_if_else_persist("zmd_kept", "zmd_restarted", "could_not_restart_zmd", "${rpm_package_check_interval}"),
                   ifvarclass => "!(zmd_kept|zmd_restarted|could_not_restart_zmd).!zypper_version_ok";

    pass3.(redhat|SuSE)::

      "any" usebundle => rudder_common_reports_generic_index("packageInstallation", "rpm_package_installation_${index_rpmpkg}", "${rpm_data[${index_rpmpkg}][3]}", "RHEL/CentOS/SuSE packages", "${rpm_package[${index_rpmpkg}]}", "RPM package ${rpm_package[${index_rpmpkg}]} with policy: ${rpm_data[${index_rpmpkg}][0]}", "${index_rpmpkg}");

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_error", "${rpm_data[${index_rpmpkg}][3]}", "RHEL/CentOS/SuSE packages", "${rpm_package[${index_rpmpkg}]}", "Error taking action for RPM package ${rpm_package[${index_rpmpkg}]} with policy: ${rpm_data[${index_rpmpkg}][0]} - could not fetch the version to update", "${index_rpmpkg}"),
        ifvarclass => "is_update_action_${index_rpmpkg}.!${index_rpmpkg}_package_version_defined.rpm_package_installation_updated_available_packages_error";

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_success", "${rpm_data[${index_rpmpkg}][3]}", "RHEL/CentOS/SuSE packages", "${rpm_package[${index_rpmpkg}]}", "No action required for RPM package ${rpm_package[${index_rpmpkg}]} with policy: ${rpm_data[${index_rpmpkg}][0]}", "${index_rpmpkg}"),
        ifvarclass => "is_update_action_${index_rpmpkg}.!${index_rpmpkg}_package_version_defined.(rpm_package_installation_updated_available_packages_kept|rpm_package_installation_updated_available_packages_repaired).!${index_rpmpkg}_update_version_valid";

      # Post hooks reporting
      "any" usebundle => rudder_common_report_index("packageInstallation", "result_repaired", "${rpm_data[${index_rpmpkg}][3]}", "Post-modification hook", "${rpm_package[${index_rpmpkg}]}", "Successfully executed post hook (${rpm_posthook[${index_rpmpkg}]}) for RPM package ${rpm_package[${index_rpmpkg}]}", "${index_rpmpkg}"),
        ifvarclass => "rpm_posthook_${index_rpmpkg}_repaired";

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_error", "${rpm_data[${index_rpmpkg}][3]}", "Post-modification hook", "${rpm_package[${index_rpmpkg}]}", "Error executing post hook (${rpm_posthook[${index_rpmpkg}]}) for RPM package ${rpm_package[${index_rpmpkg}]}", "${index_rpmpkg}"),
        ifvarclass => "rpm_posthook_${index_rpmpkg}_error";

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_error", "${rpm_data[${index_rpmpkg}][3]}", "Post-modification hook", "${rpm_package[${index_rpmpkg}]}", "Can't executing post hook (${rpm_posthook[${index_rpmpkg}]}) for RPM package ${rpm_package[${index_rpmpkg}]} as policy ${rpm_data[${index_rpmpkg}][0]} failed", "${index_rpmpkg}"),
        ifvarclass => "rpm_package_installation_${index_rpmpkg}_error.execute_command_${index_rpmpkg}";

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_success", "${rpm_data[${index_rpmpkg}][3]}", "Post-modification hook", "${rpm_package[${index_rpmpkg}]}", "No hook executed since package promise was kept", "${index_rpmpkg}"),
        ifvarclass => "rpm_package_installation_${index_rpmpkg}_kept.execute_command_${index_rpmpkg}";

      "any" usebundle => rudder_common_report_index("packageInstallation", "result_na", "${rpm_data[${index_rpmpkg}][3]}", "Post-modification hook", "${rpm_package[${index_rpmpkg}]}", "No post hook execution set for RPM package ${rpm_package[${index_rpmpkg}]}", "${index_rpmpkg}"),
        ifvarclass => "!execute_command_${index_rpmpkg}";

      

    # ZMD Process presence related reports
    pass3.!(SuSE_10.!zypper_version_ok)::
      "any" usebundle => rudder_common_report("packageInstallation", "result_na", "${rpm_data[${index_rpmpkg}][3]}", "ZMD process", "None", "ZMD process presence verification not necessary. Skipping ...");

    pass3.SuSE_10.!zypper_version_ok.!zmd_restart::
      "any" usebundle => rudder_common_report("packageInstallation", "result_success", "${rpm_data[${index_rpmpkg}][3]}", "ZMD process", "None", "The ZMD process is present. Skipping ...");

    pass3.SuSE_10.!zypper_version_ok.!service_restart_novell_zmd_not_ok.service_restart_novell_zmd_ok::
      "any" usebundle => rudder_common_report("packageInstallation", "result_repaired", "${rpm_data[${index_rpmpkg}][3]}", "ZMD process", "None", "The ZMD daemon was successfully restarted");

    pass3.SuSE_10.!zypper_version_ok.service_restart_novell_zmd_not_ok::
      "any" usebundle => rudder_common_report("packageInstallation", "result_error", "${rpm_data[${index_rpmpkg}][3]}", "ZMD process", "None", "The ZMD daemon failed to restart");

  processes:

    SuSE_10.!zypper_version_ok::

      "/usr/lib/zmd/zmd.exe"
        restart_class => "zmd_restart";

  commands:
    "${rpm_posthook[${index_rpmpkg}]}"
      classes => rudder_common_classes("rpm_posthook_${index_rpmpkg}"),
      contain => in_shell,
      ifvarclass => "execute_command_${index_rpmpkg}.rpm_package_installation_${index_rpmpkg}_repaired.!rpm_package_installation_${index_rpmpkg}_error";

}

body package_method generic_nobulk(rpm_pkg_timeout) {


    package_version_less_command  => "${rudder_rpm_knowledge.rpm_compare_less}";
    package_version_equal_command => "${rudder_rpm_knowledge.rpm_compare_equal}";

    package_changes => "individual";

  redhat::
    package_list_command          => "/bin/rpm -qa --qf '%{name} %{version}-%{release} %{arch}\n'";
    package_list_name_regex       => "^(\S+?)\s\S+?\s\S+$";
    package_list_version_regex    => "^\S+?\s(\S+?)\s\S+$";
    package_list_arch_regex       => "^\S+?\s\S+?\s(\S+)$";
    package_installed_regex       => ".*";
    package_name_convention       => "${name}-${version}";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_add_command           => "/usr/bin/yum ${redhat_knowledge.yum_options} -y install";
    package_delete_command        => "/bin/rpm -e";
    package_update_command        => "/usr/bin/yum ${redhat_knowledge.yum_options} -y update";
    package_verify_command        => "/bin/rpm -V";


  zypper_version_ok::
    package_list_command => "/bin/rpm -qa --queryformat \"i | repos | %{name} | %{version}-%{release} | %{arch}\n\"";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_installed_regex => "i.*";
    package_list_name_regex => "[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_version_regex => "[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_arch_regex => "[^|]+\|[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_name_convention => "${name}";
    package_add_command => "/usr/bin/zypper --non-interactive install";
    package_delete_command => "/usr/bin/zypper --non-interactive remove --force-resolution";
    package_update_command => "/usr/bin/zypper --non-interactive update";
    package_verify_command => "/usr/bin/zypper --non-interactive verify$";
  
  SuSE_10.!zypper_version_ok::
    package_list_command => "/bin/rpm -qa --queryformat \"i | repos | %{name} | %{version}-%{release} | %{arch}\n\"";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_installed_regex => "i.*";
    package_list_name_regex    => "[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_version_regex => "[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_arch_regex    => "[^|]+\|[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_name_convention => "${name}";
    package_add_command => "/usr/bin/rug install -y";
    package_delete_command => "/usr/bin/rug remove -y";
    package_update_command => "/usr/bin/rug update -y";
    package_verify_command => "/usr/bin/rug verify -y$"; # $ means no args
}

body package_method generic_nobulk_version(rpm_pkg_timeout) {


    package_version_less_command  => "${rudder_rpm_knowledge.rpm_compare_less}";
    package_version_equal_command => "${rudder_rpm_knowledge.rpm_compare_equal}";

    package_changes => "individual";

  redhat::
    package_list_command          => "/bin/rpm -qa --qf '%{name} %{version}-%{release} %{arch}\n'";
    package_list_name_regex       => "^(\S+?)\s\S+?\s\S+$";
    package_list_version_regex    => "^\S+?\s(\S+?)\s\S+$";
    package_list_arch_regex       => "^\S+?\s\S+?\s(\S+)$";
    package_installed_regex       => ".*";
    package_name_convention       => "${name}-${version}";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_add_command           => "/usr/bin/yum ${redhat_knowledge.yum_options} -y install";
    package_delete_command        => "/bin/rpm -e";
    package_update_command        => "/usr/bin/yum ${redhat_knowledge.yum_options} -y update";
    package_verify_command        => "/bin/rpm -V";


  zypper_version_ok::
    package_list_command => "/bin/rpm -qa --queryformat \"i | repos | %{name} | %{version}-%{release} | %{arch}\n\"";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_installed_regex => "i.*";
    package_list_name_regex => "[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_version_regex => "[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_arch_regex => "[^|]+\|[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_name_convention => "${name}=${version}";
    package_add_command => "/usr/bin/zypper --non-interactive install";
    package_delete_command => "/usr/bin/zypper --non-interactive remove --force-resolution";
    package_update_command => "/usr/bin/zypper --non-interactive update";
    package_verify_command => "/usr/bin/zypper --non-interactive verify$";

  SuSE_10.!zypper_version_ok::
    package_list_command => "/bin/rpm -qa --queryformat \"i | repos | %{name} | %{version}-%{release} | %{arch}\n\"";
    package_list_update_ifelapsed => "${rpm_pkg_timeout}";
    package_installed_regex => "i.*";
    package_list_name_regex    => "[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_version_regex => "[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_list_arch_regex    => "[^|]+\|[^|]+\|[^|]+\|[^|]+\|\s+([^\s]+).*";
    package_name_convention => "${name}";
    package_add_command => "/usr/bin/rug install -y";
    package_delete_command => "/usr/bin/rug remove -y";
    package_update_command => "/usr/bin/rug update -y";
    package_verify_command => "/usr/bin/rug verify -y$"; # $ means no args
}
