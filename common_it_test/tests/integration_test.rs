use indoc::indoc;
use yaml_rust2::YamlLoader;
use mvv_common_it_test::docker_compose_util::remove_host_ports_in_docker_compose_yaml;
use mvv_common_it_test::yaml::yaml_to_string;
use mvv_common::test::TestResultUnwrap;
//--------------------------------------------------------------------------------------------------



/// It is not real INTEGRATION test, it is just test of module 'integration'.
//--------------------------------------------------------------------------------------------------

#[test]
fn remove_host_ports_in_docker_compose_yaml_01_test() {

    let yaml_1 = indoc! {"
        networks:
          rust-account-soa-net:
            name: account-soa-it-tests
        services:
          rust-account-soa:
            # comment 01
            image: mvv_rust_account_soa${DOCKER_IMAGE_PROFILE_SUFFIX}
            networks:
              - rust-account-soa-net
            environment:
              # - JAVA_TOOL_OPTIONS=-agentlib:jdwp=transport=dt_socket,address=*:8000,server=y,suspend=n
              - SERVER_PORT=8080
            ports:
              # DOCKER_HOST_ACCOUNT_SOA_PORT_WITH_COLON=8095:
              - 8101:8080
              - 8089
              - 127.0.0.1:3901:3901
    "};

    let mut yaml_docs = YamlLoader::load_from_str(yaml_1).test_unwrap();
    let yaml = &mut yaml_docs[0];

    let changed = remove_host_ports_in_docker_compose_yaml(yaml).test_unwrap();

    assert_eq!(changed, true);

    let changed_yaml = yaml_to_string(yaml).test_unwrap();

    let expected = indoc! {"
            ---
            networks:
              rust-account-soa-net:
                name: account-soa-it-tests
            services:
              rust-account-soa:
                image: \"mvv_rust_account_soa${DOCKER_IMAGE_PROFILE_SUFFIX}\"
                networks:
                  - rust-account-soa-net
                environment:
                  - SERVER_PORT=8080
                ports:
                  - 8080
                  - 8089
                  - 3901
            "}.trim();

    assert_eq!(changed_yaml.trim(), expected);
}
