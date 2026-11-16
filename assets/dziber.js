export function start() {
    wasm.start();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_debug_string_ab4b34d23d6778bd: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_3baa9db1a987f47d: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_63322ec0cd6ea4ef: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_6df3bf7ef1164ed3: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_29a43b4d42920abd: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_string_get_7ed5322991caaec5: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_b46c9b5a9f08ec37: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_addEventListener_79f868f51ae88579: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_addEventListener_8176dab41b09531c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_addIceCandidate_ca41452441700546: function(arg0, arg1) {
            const ret = arg0.addIceCandidate(arg1);
            return ret;
        },
        __wbg_addStream_aeddecb12c755808: function(arg0, arg1) {
            arg0.addStream(arg1);
        },
        __wbg_appendChild_e95c8b3b936d250a: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_call_a24592a6f349a97e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_candidate_770f2da8df8e2fd4: function(arg0, arg1) {
            const ret = arg1.candidate;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_candidate_b4c2dd5b9ce044c8: function(arg0) {
            const ret = arg0.candidate;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_clearTimeout_113b1cde814ec762: function(arg0) {
            const ret = clearTimeout(arg0);
            return ret;
        },
        __wbg_close_88106990eea7f544: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_close_c2f057da49646ced: function(arg0) {
            arg0.close();
        },
        __wbg_closest_7cf140b67f393018: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.closest(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_code_c4f315d8dc91de14: function(arg0) {
            const ret = arg0.code;
            return ret;
        },
        __wbg_createAnswer_c3b85ae3f2b2063c: function(arg0) {
            const ret = arg0.createAnswer();
            return ret;
        },
        __wbg_createElement_bbd4c90086fe6f7b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createOffer_5d9288d2b3b397b6: function(arg0) {
            const ret = arg0.createOffer();
            return ret;
        },
        __wbg_crypto_38df2bab126b63dc: function(arg0) {
            const ret = arg0.crypto;
            return ret;
        },
        __wbg_data_bb9dffdd1e99cf2d: function(arg0) {
            const ret = arg0.data;
            return ret;
        },
        __wbg_dispatchEvent_d8729fc7c0462621: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.dispatchEvent(arg1);
            return ret;
        }, arguments); },
        __wbg_document_7a41071f2f439323: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_error_2001591ad2463697: function(arg0) {
            console.error(arg0);
        },
        __wbg_fetch_8d9b732df7467c44: function(arg0) {
            const ret = fetch(arg0);
            return ret;
        },
        __wbg_forEach_99add773fc5cd5e6: function(arg0, arg1, arg2) {
            try {
                var state0 = {a: arg1, b: arg2};
                var cb0 = (arg0, arg1, arg2) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__wasm_bindgen_93f5688e3dc4377b___JsValue__js_sys_29e60c75b17a24ed___Set______true_(a, state0.b, arg0, arg1, arg2);
                    } finally {
                        state0.a = a;
                    }
                };
                arg0.forEach(cb0);
            } finally {
                state0.a = 0;
            }
        },
        __wbg_getAttribute_8627dea35cdb7b06: function(arg0, arg1, arg2, arg3) {
            const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getElementById_0b5a508c91194690: function(arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getHours_bba0ffaba65cf3f1: function(arg0) {
            const ret = arg0.getHours();
            return ret;
        },
        __wbg_getMinutes_240bbdd69fb6e5d0: function(arg0) {
            const ret = arg0.getMinutes();
            return ret;
        },
        __wbg_getRandomValues_c44a50d8cfdaebeb: function() { return handleError(function (arg0, arg1) {
            arg0.getRandomValues(arg1);
        }, arguments); },
        __wbg_getTracks_a2bdf7c9bc3f23c7: function(arg0) {
            const ret = arg0.getTracks();
            return ret;
        },
        __wbg_getUserMedia_35013b9629195e5e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getUserMedia(arg1);
            return ret;
        }, arguments); },
        __wbg_get_6011fa3a58f61074: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_8360291721e2339f: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_sdp_9d67039437218975: function(arg0, arg1) {
            const ret = arg1.sdp;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_instanceof_ArrayBuffer_7c8433c6ed14ffe3: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Element_56c8d987654f359e: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Error_6872d63ba7922898: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlAudioElement_c2affb8eafa47d12: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLAudioElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlButtonElement_56ddb0781cfac72e: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLButtonElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlFormElement_a367f46afa5cac5f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLFormElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_8dc30e795ec4f2a5: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlTextAreaElement_73ab48e7517a68c2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTextAreaElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_MediaStreamTrack_d53a1874bca5727c: function(arg0) {
            let result;
            try {
                result = arg0 instanceof MediaStreamTrack;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_MediaStream_f32fa090e75434a3: function(arg0) {
            let result;
            try {
                result = arg0 instanceof MediaStream;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Response_9b2d111407865ff2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_cc64c86c8ef9e02b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_key_2cbc38fa83cdb336: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_9f1775224cf1d815: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_mediaDevices_b9d943bd4a64962e: function() { return handleError(function (arg0) {
            const ret = arg0.mediaDevices;
            return ret;
        }, arguments); },
        __wbg_message_cb4f84ee66e5e341: function(arg0) {
            const ret = arg0.message;
            return ret;
        },
        __wbg_msCrypto_bd5a034af96bcba6: function(arg0) {
            const ret = arg0.msCrypto;
            return ret;
        },
        __wbg_name_d3c35622d14bb080: function(arg0) {
            const ret = arg0.name;
            return ret;
        },
        __wbg_navigator_bc077756492232c5: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_0c7403db6e782f19: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_15a4889b4b90734d: function() { return handleError(function () {
            const ret = new Headers();
            return ret;
        }, arguments); },
        __wbg_new_4331d3ba658c037d: function() { return handleError(function () {
            const ret = new URLSearchParams();
            return ret;
        }, arguments); },
        __wbg_new_490db15a0a09fb24: function() { return handleError(function (arg0, arg1) {
            const ret = new URL(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_682678e2f47e32bc: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_7913666fe5070684: function(arg0) {
            const ret = new Date(arg0);
            return ret;
        },
        __wbg_new_aa8d0fa9762c29bd: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_with_configuration_83fa420a48d71a6d: function() { return handleError(function (arg0) {
            const ret = new RTCPeerConnection(arg0);
            return ret;
        }, arguments); },
        __wbg_new_with_event_init_dict_296b48bdb72e7520: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new CloseEvent(getStringFromWasm0(arg0, arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_new_with_length_8c854e41ea4dae9b: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_new_with_str_5f3ca98523ee76ef: function() { return handleError(function (arg0, arg1) {
            const ret = new Request(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_with_str_76229cfd7be7bd04: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = new WebSocket(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        }, arguments); },
        __wbg_new_with_str_and_init_897be1708e42f39d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_node_84ea875411254db1: function(arg0) {
            const ret = arg0.node;
            return ret;
        },
        __wbg_now_a9b7df1cbee90986: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_ok_f7783a2e6ac7fe17: function(arg0) {
            const ret = arg0.ok;
            return ret;
        },
        __wbg_parse_30c988d89869cab2: function(arg0, arg1) {
            const ret = Date.parse(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_preventDefault_f55c01cb5fd2bcc0: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_process_44c7a14e11e9f69e: function(arg0) {
            const ret = arg0.process;
            return ret;
        },
        __wbg_prototypesetcall_a6b02eb00b0f4ce2: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_push_471a5b068a5295f6: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_querySelector_12b6c7cdf26a3483: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_queueMicrotask_5d15a957e6aa920e: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_f8819e5ffc402f36: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_randomFillSync_6c25eac9869eb53c: function() { return handleError(function (arg0, arg1) {
            arg0.randomFillSync(arg1);
        }, arguments); },
        __wbg_readyState_c78e609c7de3b381: function(arg0) {
            const ret = arg0.readyState;
            return ret;
        },
        __wbg_reason_e943590a4ef0d587: function(arg0, arg1) {
            const ret = arg1.reason;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_removeAttribute_c75ac657c944b3f1: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_removeEventListener_7bdf07404d9b24bd: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_require_b4edbdcf3e2a1ef0: function() { return handleError(function () {
            const ret = module.require;
            return ret;
        }, arguments); },
        __wbg_reset_09b7c2d924c3010d: function(arg0) {
            arg0.reset();
        },
        __wbg_resolve_e6c466bc1052f16c: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_scrollHeight_67ba70eba759d3f0: function(arg0) {
            const ret = arg0.scrollHeight;
            return ret;
        },
        __wbg_sdpMLineIndex_9eeb1e97bcdbea9d: function(arg0) {
            const ret = arg0.sdpMLineIndex;
            return isLikeNone(ret) ? 0xFFFFFF : ret;
        },
        __wbg_sdpMid_0be5d24b5eec5543: function(arg0, arg1) {
            const ret = arg1.sdpMid;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_search_98479a9dd6b1643e: function(arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_send_15358dbe221c6258: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.send(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_send_186c85704c7f2d00: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.send(getArrayU8FromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_setAttribute_6fde4098d274155c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setLocalDescription_f325e5e4b975443c: function(arg0, arg1) {
            const ret = arg0.setLocalDescription(arg1);
            return ret;
        },
        __wbg_setRemoteDescription_808ec45573e321da: function(arg0, arg1) {
            const ret = arg0.setRemoteDescription(arg1);
            return ret;
        },
        __wbg_setTimeout_ef24d2fc3ad97385: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_set_1ffc463d4c541483: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_set_audio_bool_511bead69975aa52: function(arg0, arg1) {
            arg0.audio = arg1 !== 0;
        },
        __wbg_set_binaryType_770e68648ca5e83d: function(arg0, arg1) {
            arg0.binaryType = __wbindgen_enum_BinaryType[arg1];
        },
        __wbg_set_body_be11680f34217f75: function(arg0, arg1) {
            arg0.body = arg1;
        },
        __wbg_set_candidate_4e0df3162610f37e: function(arg0, arg1, arg2) {
            arg0.candidate = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_className_1c8245364f86eb8d: function(arg0, arg1, arg2) {
            arg0.className = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_code_10171c89681092ae: function(arg0, arg1) {
            arg0.code = arg1;
        },
        __wbg_set_disabled_62644deb31d137bb: function(arg0, arg1) {
            arg0.disabled = arg1 !== 0;
        },
        __wbg_set_disabled_8dab3b8eede06d80: function(arg0, arg1) {
            arg0.disabled = arg1 !== 0;
        },
        __wbg_set_disabled_a78da97db89c031f: function(arg0, arg1) {
            arg0.disabled = arg1 !== 0;
        },
        __wbg_set_headers_50fc01786240a440: function(arg0, arg1) {
            arg0.headers = arg1;
        },
        __wbg_set_ice_servers_cc143896b6f29661: function(arg0, arg1) {
            arg0.iceServers = arg1;
        },
        __wbg_set_method_c9f1f985f6b6c427: function(arg0, arg1, arg2) {
            arg0.method = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_once_e747a93482f65a72: function(arg0, arg1) {
            arg0.once = arg1 !== 0;
        },
        __wbg_set_onicecandidate_39803e5d172e0d96: function(arg0, arg1) {
            arg0.onicecandidate = arg1;
        },
        __wbg_set_ontrack_aa75230c1dbce5b6: function(arg0, arg1) {
            arg0.ontrack = arg1;
        },
        __wbg_set_reason_c982e157c54cb4cb: function(arg0, arg1, arg2) {
            arg0.reason = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_scrollTop_9de7da53be65a3ce: function(arg0, arg1) {
            arg0.scrollTop = arg1;
        },
        __wbg_set_sdp_1b3512f1d4d75443: function(arg0, arg1, arg2) {
            arg0.sdp = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_sdp_m_line_index_76cc4410489904ec: function(arg0, arg1) {
            arg0.sdpMLineIndex = arg1 === 0xFFFFFF ? undefined : arg1;
        },
        __wbg_set_sdp_mid_612df723a380cb7a: function(arg0, arg1, arg2) {
            arg0.sdpMid = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_search_2982fabf5212b32d: function(arg0, arg1, arg2) {
            arg0.search = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_srcObject_41746555dc5245c0: function(arg0, arg1) {
            arg0.srcObject = arg1;
        },
        __wbg_set_textContent_223eb7313f8a7178: function(arg0, arg1, arg2) {
            arg0.textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_e681927393d674ce: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_RtcSdpType[arg1];
        },
        __wbg_set_urls_41b10d1a5b14aca8: function(arg0, arg1) {
            arg0.urls = arg1;
        },
        __wbg_set_value_d3cd5a0ce61506cd: function(arg0, arg1, arg2) {
            arg0.value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_video_bool_c57f90d4727efc7d: function(arg0, arg1) {
            arg0.video = arg1 !== 0;
        },
        __wbg_shiftKey_e483c13c966878f6: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_static_accessor_GLOBAL_8cfadc87a297ca02: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_602256ae5c8f42cf: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_e445c1c7484aecc3: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f20e8576ef1e0f17: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_stop_d7613f5d83ba5c45: function(arg0) {
            arg0.stop();
        },
        __wbg_streams_097b3473ed86b90c: function(arg0) {
            const ret = arg0.streams;
            return ret;
        },
        __wbg_subarray_f8ca46a25b1f5e0d: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_target_6d97e221d11b71b6: function(arg0) {
            const ret = arg0.target;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_then_792e0c862b060889: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_8e16ee11f05e4827: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_toString_306ed0b9f320c1ca: function(arg0) {
            const ret = arg0.toString();
            return ret;
        },
        __wbg_toString_6dc1a94e0bdba378: function(arg0) {
            const ret = arg0.toString();
            return ret;
        },
        __wbg_url_94ef047be3ab790a: function(arg0, arg1) {
            const ret = arg1.url;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_value_6079dd28568d83c9: function(arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_value_acdc3fed0dd155ba: function(arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_versions_276b2795b1c6a219: function(arg0) {
            const ret = arg0.versions;
            return ret;
        },
        __wbg_wasClean_bd109e45fffa711a: function(arg0) {
            const ret = arg0.wasClean;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 353, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__core_d532f8654abfce50___result__Result_____wasm_bindgen_93f5688e3dc4377b___JsError___true_);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("CloseEvent")], shim_idx: 191, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 191, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__2);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 117, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MessageEvent")], shim_idx: 191, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__4);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("RTCPeerConnectionIceEvent")], shim_idx: 117, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__5);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("RTCTrackEvent")], shim_idx: 117, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__6);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 167, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 195, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true__1_);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./dziber_bg.js": import0,
    };
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true_(arg0, arg1) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true_(arg0, arg1);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true__1_(arg0, arg1) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke_______true__1_(arg0, arg1);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true_(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__2(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__2(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true_(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__4(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_CloseEvent__CloseEvent______true__4(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__5(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__5(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__6(arg0, arg1, arg2) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___web_sys_1f513fa4b39d43d0___features__gen_KeyboardEvent__KeyboardEvent______true__6(arg0, arg1, arg2);
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__core_d532f8654abfce50___result__Result_____wasm_bindgen_93f5688e3dc4377b___JsError___true_(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__core_d532f8654abfce50___result__Result_____wasm_bindgen_93f5688e3dc4377b___JsError___true_(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__wasm_bindgen_93f5688e3dc4377b___JsValue__js_sys_29e60c75b17a24ed___Set______true_(arg0, arg1, arg2, arg3, arg4) {
    wasm.wasm_bindgen_93f5688e3dc4377b___convert__closures_____invoke___wasm_bindgen_93f5688e3dc4377b___JsValue__wasm_bindgen_93f5688e3dc4377b___JsValue__js_sys_29e60c75b17a24ed___Set______true_(arg0, arg1, arg2, arg3, arg4);
}


const __wbindgen_enum_BinaryType = ["blob", "arraybuffer"];


const __wbindgen_enum_RtcSdpType = ["offer", "pranswer", "answer", "rollback"];

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('dziber_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
