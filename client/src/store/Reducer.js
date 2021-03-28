import * as type from './ActionTypes';
import initialState from './InitialState';

export default function reducer(state = initialState, action) {
  switch (action.type) {
    case type.SET_KEPLR_ENABLED:
      return {
        ...state,
        keplrEnabled: action.keplrEnabled,
      };
    case type.SET_SCRT_AUTHORIZED:
      return {
        ...state,
        scrtAuthorized: action.scrtAuthorized,
      };
    case type.SET_SECRET_CLIENT:
      return {
        ...state,
        secretClient: action.secretClient,
      };
    case type.SET_VIEWING_KEY:
      return {
        ...state,
        viewingKey: action.viewingKey,
      };
    case type.SET_LOGGED_IN:
      return {
        ...state,
        loggedIn: action.loggedIn,
      };
    case type.SET_HANDLE_SEARCH_FIELD_VALUE:
      return {
        ...state,
        handleSearchFieldValue: action.handleSearchFieldValue,
      };
    case type.SET_DRAWER_MENU_SELECTION:
      return {
        ...state,
        drawerMenuSelection: action.drawerMenuSelection,
      };
    case type.SET_FARDEL_VIEW_DIALOG_OPEN:
      return {
        ...state,
        fardelViewDialogOpen: action.fardelViewDialogOpen,
      };
    // View handle
    case type.SET_VIEWED_HANDLE:
      return {
        ...state,
        viewedHandle: action.viewedHandle,
      };
    case type.SET_VIEWED_HANDLE_IMG:
      return {
        ...state,
        viewedHandleImg: action.viewedHandleImg,
      };
    case type.SET_VIEWED_FARDELS:
      return {
        ...state,
        viewedFardels: action.viewedFardels,
      };
    case type.SET_SELECTED_FARDEL:
      return {
        ...state,
        selectedFardel: action.selectedFardel,
      }
    // Following
    case type.SET_FOLLOWING:
      return {
        ...state,
        following: action.following,
      };
    // My Fardels
    case type.SET_NEW_FARDEL_MESSAGE:
      return {
        ...state,
        newFardelMessage: action.newFardelMessage,
      };
    case type.SET_NEW_FARDEL_CONTENTS:
      return {
        ...state,
        newFardelContents: action.newFardelContents,
      };
    case type.SET_NEW_FARDEL_THUMBNAIL:
      return {
        ...state,
        newFardelThumbnail: action.newFardelThumbnail,
      };
    case type.CLEAR_NEW_FARDEL:
      return {
        ...state,
        newFardelMessage: '',
        newFardelContents: { contentsText: '', cost: '', ipfsCid: '', passphrase: ''},
      };
    // Profile
    case type.SET_HANDLE:
      return {
        ...state,
        handle: action.handle
      };
    case type.SET_PROFILE_DESCRIPTION:
      return {
        ...state,
        profileDescription: action.profileDescription
      };
    case type.SET_PROFILE_THUMBNAIL:
      return {
        ...state,
        profileThumbnail: action.profileThumbnail
      };
    default:
      return { ...state };
  }
}
