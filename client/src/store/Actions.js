import * as type from './ActionTypes'

const contractAddress = process.env.REACT_APP_ContractAddress;
const utf8decoder = new TextDecoder();

export function setKeplrEnabled(keplrEnabled) {
  return {
    type: type.SET_KEPLR_ENABLED,
    keplrEnabled
  }
}

export function setScrtAuthorized(scrtAuthorized) {
  return {
    type: type.SET_SCRT_AUTHORIZED,
    scrtAuthorized
  }
}

export function setSecretClient(secretClient) {
  return {
    type: type.SET_SECRET_CLIENT,
    secretClient
  }
}

export function setViewingKey(viewingKey) {
  return {
    type: type.SET_VIEWING_KEY,
    viewingKey
  }
}

export function setLoggedIn(loggedIn) {
  return {
    type: type.SET_LOGGED_IN,
    loggedIn,
  }
}

export function setHandleSearchFieldValue(handleSearchFieldValue) {
  return {
    type: type.SET_HANDLE_SEARCH_FIELD_VALUE,
    handleSearchFieldValue
  }
}

export function setDrawerMenuSelection(drawerMenuSelection) {
  return {
    type: type.SET_DRAWER_MENU_SELECTION,
    drawerMenuSelection
  }
}

export function setFardelViewDialogOpen(fardelViewDialogOpen) {
  return {
    type: type.SET_FARDEL_VIEW_DIALOG_OPEN,
    fardelViewDialogOpen
  }
}

// View handle
export function setViewedHandle(viewedHandle) {
  return {
    type: type.SET_VIEWED_HANDLE,
    viewedHandle
  }
}

export function setViewedHandleImg(viewedHandleImg) {
  return {
    type: type.SET_VIEWED_HANDLE_IMG,
    viewedHandleImg
  }
}

export function setViewedFardels(viewedFardels) {
  return {
    type: type.SET_VIEWED_FARDELS,
    viewedFardels
  }
}

export function setSelectedFardel(selectedFardel) {
  return {
    type: type.SET_SELECTED_FARDEL,
    selectedFardel,
  }
}

// Following

export function setFollowing(following) {
  return {
    type: type.SET_FOLLOWING,
    following
  }
}

export function startFollowing(handle) {
  return function (dispatch, getState) {
    const following = getState().following;
    if (following.indexOf(handle) === -1) {
      dispatch(setFollowing([...following, handle]));
    }
    return Promise.resolve()
  }
}

export function stopFollowing(handle) {
  return function (dispatch, getState) {
    const following = getState().following;
    const index = following.indexOf(handle);
    if (index !== -1) {
      dispatch(setFollowing(
        [ 
          ...following.slice(0, index), 
          ...following.slice(index+1, -1)
        ]
      ));
    }
    return Promise.resolve()
  }
}

// My fardels
export function setNewFardelMessage(newFardelMessage) {
  return {
    type: type.SET_NEW_FARDEL_MESSAGE,
    newFardelMessage
  }
}

export function setNewFardelContents(newFardelContents) {
  return {
    type: type.SET_NEW_FARDEL_CONTENTS,
    newFardelContents
  }
}

export function setNewFardelThumbnail(newFardelThumbnail) {
  return {
    type: type.SET_NEW_FARDEL_THUMBNAIL,
    newFardelThumbnail
  }
}

export function clearNewFardel() {
  return {
    type: type.CLEAR_NEW_FARDEL,
    clearNewFardel
  }
}

// Profile
export function setHandle(handle) {
  return {
    type: type.SET_HANDLE,
    handle
  }
}

export function setProfileDescription(profileDescription) {
  return {
    type: type.SET_PROFILE_DESCRIPTION,
    profileDescription
  }
}

export function setProfileThumbnail(profileThumbnail) {
  return {
    type: type.SET_PROFILE_THUMBNAIL,
    profileThumbnail
  }
}

// get the handle for the authorized user with viewing key
export function queryGetHandle(key) {
  return async function(dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && key) {
      const { senderAddress } = secretClient;
      const queryMsg = { get_handle: { address: senderAddress, key } };
      try {
        const response = await secretClient.queryContractSmart(contractAddress, queryMsg);
        if (response.viewing_key_error) {
          console.log(response);

          dispatch(setLoggedIn(false));
        } else {
          console.log(response);

          if (!response.get_handle || !response.get_handle.handle) {
            dispatch(setLoggedIn(false));
          } else {
            dispatch(setHandle(response.get_handle.handle));
            dispatch(setLoggedIn(true));

            // get following
            dispatch(queryGetFollowing(key));

            // get fardels
            //dispatch(queryGetFardelsFromHandle(response.get_handle.handle, key, 0, 10));
          }
        }
        // TODO dispatch reponse handling

        return response;
      } catch (error) {
        // TODO dispatch on error
        return error;
      }
    }
  }
}

// get the other users you are following
export function queryGetFollowing(key) {
  return async function(dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && key) {
      const { senderAddress } = secretClient;
      const queryMsg = { get_following: { address: senderAddress, key } };
      try {
        const response = await secretClient.queryContractSmart(contractAddress, queryMsg);
        console.log(response);
        if (response.get_following && response.get_following.following) {
          dispatch(setFollowing(response.get_following.following));
        }
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch on error
        return error;
      }
    }
  }
}

// check if the given handle is available or already taken
export function queryIsHandleAvailable(handle) {
  return async function(dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && handle) {
      const queryMsg = { is_handle_available: { handle } };
      try {
        const response = await secretClient.queryContractSmart(contractAddress, queryMsg);
        let data = JSON.parse(utf8decoder.decode(response));
        console.log(data);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch on error
        return error;
      }
    }
  }
}

// get the fardels for handle
// defaults to latest 10
export function queryGetFardelsFromHandle(handle, page, pageSize) {
  return async function(dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;
    const key = getState().viewingKey;

    if (scrtAuthorized && secretClient && handle) {
      let queryMsg;
      if (key) {
        const { senderAddress } = secretClient;
        queryMsg = { 
          get_fardels_auth: { 
            address: senderAddress,
            key,
            handle,
            page, 
            page_size: pageSize,
          }
        } 
      } else {
        queryMsg = { 
          get_fardels: { 
            handle,
            page, 
            page_size: pageSize,
          }
        } 
      }
      try {
        console.log(queryMsg);
        const response = await secretClient.queryContractSmart(contractAddress, queryMsg);
        if (response.get_fardels && response.get_fardels.fardels) {
          let fardels = response.get_fardels.fardels.map( (f) => { 
            let _f = f;
            _f.cost = (parseFloat(f.cost) / 1000000).toString();
            return _f;
          });  
          dispatch(setViewedFardels(fardels));
        }
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch on error
        return error;
      }
    }
  }
}

// registers handle
// if a handle already exists for this user, replaces it
// if handle is already in use by another user returns an error
export function executeRegister(handle, description) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && handle ) {
      // check to make sure handle is not already taken
      const queryMsg = { is_handle_available: { handle } };
      try {
        const response = await secretClient.queryContractSmart(contractAddress, queryMsg);
        if (response.response) { // response is true
          // TODO dispatch reponse handling
          return response;
        }
      } catch (error) { }

      // register the new user (handle + description)
      const desc = (description === null || description === undefined) ? "" : description;
      const executeMsg = { register: { handle, description: desc } };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        // TODO dispatch reponse handling
        dispatch(setHandle(handle));
        dispatch(setProfileDescription(desc));
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// registers handle and sets a viewing key
// if a handle already exists for this user, replaces it
// if handle is already in use by another user returns an error
//  return async function (dispatch, getState) {
//    const scrtAuthorized = getState().scrtAuthorized;
//    const secretClient = getState().secretClient;
//
//    if (scrtAuthorized && secretClient && handle && key) {
//      const executeMsg = { register_and_set_viewing_key: { handle, key } };
//      try {
//        const response = await secretClient.execute(contractAddress, executeMsg);
//        // TODO dispatch reponse handling
//        return response;
//      } catch (error) {
//        // TODO dispatch error handling
//        return error;
//      }
//    }
//  }
//}

// registers handle and generates a viewing key
// if a handle already exists for this user, replaces it
// if handle is already in use by another user returns an error
//  return async function (dispatch, getState) {
//    const scrtAuthorized = getState().scrtAuthorized;
//    const secretClient = getState().secretClient;
//
//    if (scrtAuthorized && secretClient && handle && entropy) {
//      const executeMsg = { register_and_generate_viewing_key: { handle, entropy } };
//      try {
//        const response = await secretClient.execute(contractAddress, executeMsg);
//        // TODO dispatch reponse handling
//        return response;
//      } catch (error) {
//        // TODO dispatch error handling
//        return error;
//      }
//    }
//  }
//}

// sets the viewing key for the current user
export function executeSetViewingKey(key) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && key) {
      const executeMsg = { set_viewing_key: { key } };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        dispatch(setViewingKey(key));
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// generates a new viewing key for the current user
export function executeGenerateViewingKey(entropy) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && entropy) {
      const executeMsg = { generate_viewing_key: { entropy } };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// stores a 100x100px max thumbnail img for your account
export function executeSetProfileThumbnail(img) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && img) {
      // SetProfileThumbnailImg
      const executeMsg = { set_profile_thumbnail_img: { img } };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        // TODO dispatch reponse handling

        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// deactivates the account, essentially making it not visible to anyone else.
// users can still access their unpacked fardels from deactivated users.
//export function executeDeactivate() {
//  return async function (dispatch, getState) {
//    const scrtAuthorized = getState().scrtAuthorized;
//    const secretClient = getState().secretClient;
//
//    if (scrtAuthorized && secretClient) {
//      const executeMsg = { deactivate: { } };
//      try {
//        const response = await secretClient.execute(contractAddress, executeMsg);
//        // TODO dispatch reponse handling
//        return response;
//      } catch (error) {
//        // TODO dispatch error handling
//        return error;
//      }
//    }
//  }
//}

// carries a fardel with flags
export function executeCarryFardel(publicMessage, 
                                   contentsText, 
                                   ipfsCid, 
                                   passphrase,
                                   cost) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    const costVal = +cost;
    if (scrtAuthorized && 
        secretClient && 
        publicMessage && 
        (contentsText || ipfsCid) &&
        (costVal >= 0 && costVal <= 10)
    ) {
      const costUscrt = (Math.floor(costVal * 1000000)).toString();
      const executeMsg = { 
        carry_fardel: { 
          public_message: publicMessage, 
          contents_text: contentsText,
          ipfs_cid: ipfsCid,
          passphrase,
          cost: costUscrt,
        } 
      };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}


// upvotes a fardel (only works if unpacked by user)
export function executeUpvote(fardelId) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && fardelId) {
      const executeMsg = { 
        rate_fardel: { 
          fardel_id: fardelId,
          rating: true,
        } 
      };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// downvotes a fardel (only works if unpacked by user)
export function executeDownvote(fardelId) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && fardelId) {
      const executeMsg = { 
        rate_fardel: { 
          fardel_id: fardelId,
          rating: false,
        } 
      };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// comment on a fardel
export function executeComment(fardelId, comment) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && fardelId && comment) {
      const executeMsg = { 
        comment_on_fardel: { 
          fardel_id: fardelId,
          comment
        } 
      };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);
        // TODO dispatch reponse handling
        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// deletes a fardel
// the unpacked fardels are still available those who have paid
export function executeSealFardel(fardelId) {
  return function (dispatch, getState) {
    return Promise.resolve();
  }
}

// follow another user
export function executeFollow(handle) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && handle) {
      const executeMsg = { follow: { handle } };
      try {
        const response = await secretClient.execute(contractAddress, executeMsg);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);

        return response;
      } catch (error) {
        // TODO dispatch error handling
        return error;
      }
    }
  }
}

// follow another user
export function executeUnpack(fardel, idx) {
  return async function (dispatch, getState) {
    const scrtAuthorized = getState().scrtAuthorized;
    const secretClient = getState().secretClient;

    if (scrtAuthorized && secretClient && fardel) {
      const executeMsg = { unpack_fardel: { fardel_id: fardel.id } };
      const amountVal = (Math.floor(fardel.cost * 1000000));
      const funds = [{ denom: "uscrt", amount: `${amountVal}` }];
      console.log(executeMsg);
      console.log(funds);
      try {
        const response = await secretClient.execute(contractAddress, executeMsg, "", funds);
        console.log(response);
        let data = JSON.parse(utf8decoder.decode(response.data));
        console.log(data);

        return response;
      } catch (error) {
        // TODO dispatch error handling
        console.log(error);
        return error;
      }
    }
  }
}

// block another user from viewing your fardels
//export function executeBlock(handle) {
//  return function (dispatch, getState) {
//    return Promise.resolve();
//  }
//}

// unblock another user
//export function executeUnblock(handle) {
//  return function (dispatch, getState) {
//    return Promise.resolve();
//  }
//}
